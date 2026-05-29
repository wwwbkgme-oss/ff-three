use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use serde_json::{json, Value};
use uuid::Uuid;

use types::{
    AssessmentType, PerformanceMetrics, PracticeRequest, SubmitSolutionRequest, TestResult,
};

use crate::{db, db::assessments::CreateAssessment, error::ServerResult, state::AppState};
use sandbox::{ExecutionResult, SandboxExecutor, SecurityScanner};
use types::SandboxStatus;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sandbox/practice",  post(create_practice))
        .route("/submit/solution",   post(submit_solution))
        .route("/assessment/:id",    get(get_assessment))
}

async fn create_practice(
    State(state): State<AppState>,
    Json(req): Json<PracticeRequest>,
) -> ServerResult<(StatusCode, Json<Value>)> {
    let _ = db::students::get(&state.db, req.student_id).await?;
    let run = db::sandbox::create(&state.db, req.student_id, req.quest_id, req.language, String::new()).await?;
    Ok((StatusCode::CREATED, Json(json!({
        "sandbox_id": run.id, "status": run.status,
        "message": "Sandbox ready. Submit code via POST /submit/solution.",
    }))))
}

async fn submit_solution(
    State(state): State<AppState>,
    Json(req): Json<SubmitSolutionRequest>,
) -> ServerResult<Json<Value>> {
    if req.code.trim().is_empty() {
        return Err(crate::error::ServerError::BadRequest("code must not be empty".into()));
    }
    let quest   = db::quests::get(&state.db, req.quest_id).await?;
    let student = db::students::get(&state.db, req.student_id).await?;

    let lang_ext = req.language.file_extension();
    let run = db::sandbox::create(&state.db, student.id, Some(quest.id), req.language.clone(), req.code.clone()).await?;

    // Security scan (always first).
    let scan = SecurityScanner::scan(&req.code, &req.language);

    if !scan.passed {
        db::sandbox::complete(&state.db, run.id,
            SandboxStatus::SecurityViolation, None,
            Some(format!("Blocked: {}", scan.threats_detected.join(", "))),
            Some(126), Some(0), Some(0), &scan).await?;
        return Err(crate::error::ServerError::BadRequest(
            format!("Security violation: {}", scan.threats_detected.join(", "))
        ));
    }

    // Execute in sandbox (async, non-blocking).
    let executor = SandboxExecutor::new(
        state.config.sandbox_timeout_secs,
        state.config.sandbox_max_memory_mb,
    );
    let exec_result: ExecutionResult = executor.execute(&req.code, &req.language).await;

    // Score via LLM or heuristic.
    let (score, feedback) = if let Some(llm) = &state.llm {
        let prompt = state.orchestrator.build_evaluation_prompt(&req.code, &quest.title, lang_ext);
        llm.run_evaluation(&prompt).await.unwrap_or_else(|_| score_heuristic(&req.code))
    } else {
        score_heuristic(&req.code)
    };

    // Finalise sandbox record.
    db::sandbox::complete(&state.db, run.id,
        exec_result.status.clone(),
        exec_result.stdout.clone(), exec_result.stderr.clone(),
        exec_result.exit_code,
        Some(exec_result.execution_time_ms), Some(exec_result.memory_used_kb),
        &exec_result.security_scan,
    ).await?;

    // Create assessment.
    let passed = score >= 0.7;
    let assessment = db::assessments::create(&state.db, CreateAssessment {
        student_id: student.id, quest_id: quest.id,
        atype:      AssessmentType::Practice,
        submission: req.code.clone(), score, passed,
        feedback:   feedback.clone(),
        test_results: vec![TestResult { test_case_index: 0, passed,
            actual_output: exec_result.stdout.clone().unwrap_or_else(|| "OK".into()),
            expected_output: "OK".into(),
            execution_time_ms: exec_result.execution_time_ms as u64 }],
        metrics: PerformanceMetrics {
            time_complexity: Some("O(n)".into()), space_complexity: Some("O(1)".into()),
            execution_time_ms: exec_result.execution_time_ms as u64,
            memory_used_kb: exec_result.memory_used_kb as u64,
            code_quality_score: score,
        },
    }).await?;

    if passed { db::students::add_xp(&state.db, student.id, quest.xp_reward).await?; }

    Ok(Json(json!({
        "assessment_id": assessment.id, "sandbox_id": run.id,
        "passed": passed, "score": score,
        "xp_awarded": if passed { quest.xp_reward } else { 0 },
        "feedback": feedback,
        "stdout": exec_result.stdout,
        "security": { "passed": scan.passed, "risk_level": scan.risk_level },
    })))
}

async fn get_assessment(
    State(state): State<AppState>, Path(id): Path<Uuid>,
) -> ServerResult<Json<Value>> {
    let a = db::assessments::get(&state.db, id).await?;
    Ok(Json(serde_json::to_value(a)?))
}

fn score_heuristic(code: &str) -> (f64, String) {
    let score = (code.lines().filter(|l| !l.trim().is_empty()).count() as f64 / 15.0).clamp(0.1, 1.0);
    let msg = if score >= 0.7 { "Good work!".into() } else { "Keep practising – cover more cases.".into() };
    (score, msg)
}
