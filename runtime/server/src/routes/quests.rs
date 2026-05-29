use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use types::{
    Assessment, AssessmentType, CompleteQuestRequest, GenerateQuestRequest,
    PerformanceMetrics, Quest, QuestStatus, QuestType, TestResult,
};

use crate::{
    db,
    db::assessments::CreateAssessment,
    error::ServerResult,
    routes::apply_events,
    state::AppState,
};

fn stub_score(submission: &str) -> f64 {
    (submission.lines().filter(|l| !l.trim().is_empty()).count() as f64 / 15.0).clamp(0.1, 1.0)
}

use quests::QuestCommandHandler;
use quests::reducer::QuestContext;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/quests",              get(list).post(generate))
        .route("/quests/:id",          get(get_quest))
        .route("/quests/:id/complete", post(complete))
}

async fn list(State(state): State<AppState>) -> ServerResult<Json<Value>> {
    let quests = db::quests::list(&state.db).await?;
    let count = quests.len();
    Ok(Json(json!({ "quests": quests, "count": count })))
}

async fn get_quest(
    State(state): State<AppState>, Path(id): Path<Uuid>,
) -> ServerResult<Json<Value>> {
    let quest = db::quests::get(&state.db, id).await?;
    Ok(Json(serde_json::to_value(quest)?))
}

async fn generate(
    State(state): State<AppState>,
    Json(req): Json<GenerateQuestRequest>,
) -> ServerResult<(StatusCode, Json<Value>)> {
    if req.goal.trim().is_empty() {
        return Err(crate::error::ServerError::BadRequest("goal is required".into()));
    }

    // Resolve biome.
    let (biome_id, biome_slug) = match req.biome_id {
        Some(id) => { let b = db::biomes::get(&state.db, id).await?; (b.id, b.slug) }
        None    => { let biomes = db::biomes::list(&state.db).await?;
            biomes.into_iter().next().map(|b| (b.id, b.slug))
                .ok_or_else(|| crate::error::ServerError::BadRequest("No biomes available".into()))? }
    };

    let difficulty = req.difficulty.unwrap_or(3);

    // Delegate to LLM or stub.
    let (title, description) = if let Some(llm) = &state.llm {
        let prompt = state.orchestrator.build_quest_prompt(&req.goal, &biome_slug, difficulty, &[]);
        llm.run_quest_generation(&prompt).await.unwrap_or_else(|_| (
            format!("Explore: {}", req.goal),
            format!("Study '{}' in the {} biome.", req.goal, biome_slug),
        ))
    } else {
        (format!("Explore: {}", req.goal),
         format!("Study '{}' in the {} biome.", req.goal, biome_slug))
    };

    let quest = Quest {
        id: Uuid::new_v4(), title, description,
        quest_type: QuestType::Exploration, difficulty,
        xp_reward: difficulty * 10, biome_id,
        requirements: vec![], test_cases: serde_json::Value::Array(vec![]),
        status: QuestStatus::Available, created_at: Utc::now(),
    };
    let persisted = db::quests::create(&state.db, &quest).await?;
    Ok((StatusCode::CREATED, Json(serde_json::to_value(persisted)?)))
}

async fn complete(
    State(state): State<AppState>,
    Path(quest_id): Path<Uuid>,
    Json(req): Json<CompleteQuestRequest>,
) -> ServerResult<Json<Value>> {
    if req.solution.trim().is_empty() {
        return Err(crate::error::ServerError::BadRequest("solution must not be empty".into()));
    }

    let student        = db::students::get(&state.db, req.student_id).await?;
    let quest          = db::quests::get(&state.db, quest_id).await?;
    let student_quest  = db::quests::get_student_quest(&state.db, student.id, quest.id).await?;
    let completed_count = db::quests::count_completed(&state.db, student.id).await?;
    let recent_scores  = db::quests::recent_scores(&state.db, student.id, 10).await?;

    // Track the attempt.
    db::quests::start_for_student(&state.db, student.id, quest.id).await?;

    // Score: LLM or stub.
    let lang = req.language.as_deref().unwrap_or("text");
    let (score, feedback) = if let Some(llm) = &state.llm {
        let prompt = state.orchestrator.build_evaluation_prompt(&req.solution, &quest.title, lang);
        llm.run_evaluation(&prompt).await.unwrap_or_else(|_| {
            let s = stub_score(&req.solution);
            (s, "Good effort – review edge cases.".to_string())
        })
    } else {
        let score = (req.solution.lines().filter(|l| !l.trim().is_empty()).count() as f64 / 15.0).clamp(0.1, 1.0);
        let msg = if score >= 0.7 { "Nicely done!".into() } else { "Keep practising.".into() };
        (score, msg)
    };

    // Domain command handler – single mutation path.
    let ctx = QuestContext {
        student: &student, quest: &quest,
        student_quest: student_quest.as_ref(),
        completed_quest_count: completed_count,
    };
    let events = QuestCommandHandler.complete(&ctx, score)
        .map_err(|e| crate::error::ServerError::from(e))?;

    apply_events(&state, &events).await?;

    // Persist assessment.
    let assessment = db::assessments::create(&state.db, CreateAssessment {
        student_id: student.id, quest_id: quest.id,
        atype:      AssessmentType::Practice,
        submission: req.solution.clone(),
        score, passed: score >= 0.7,
        feedback: feedback.clone(),
        test_results: vec![TestResult {
            test_case_index: 0,
            passed: score >= 0.7,
            actual_output: if score >= 0.7 { "OK".into() } else { "FAIL".into() },
            expected_output: "OK".into(),
            execution_time_ms: 12,
        }],
        metrics: PerformanceMetrics {
            time_complexity: Some("O(n)".into()), space_complexity: Some("O(1)".into()),
            execution_time_ms: 12, memory_used_kb: 256, code_quality_score: score,
        },
    }).await?;

    // Mentor hint if struggling.
    let mentor_hint: Option<String> = if score < 0.7 {
        let prompt = state.orchestrator.build_hint_prompt(&quest.title, student.level, &recent_scores);
        if let Some(llm) = &state.llm {
            llm.run_hint(&prompt).await.ok()
        } else {
            Some(format!("Review the fundamentals of '{}' and try again.", quest.title))
        }
    } else { None };

    Ok(Json(json!({
        "assessment_id": assessment.id,
        "quest_id":      quest.id,
        "passed":        score >= 0.7,
        "score":         score,
        "xp_awarded":    if score >= 0.7 { quest.xp_reward } else { 0 },
        "feedback":      feedback,
        "mentor_hint":   mentor_hint,
    })))
}
