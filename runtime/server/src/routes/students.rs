//! Student management routes.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use events::AcademyEvent;

use crate::{db, error::ServerResult, routes::apply_events, state::AppState};
use types::{EnrollRequest, QuestStatus, UpdateGoalsRequest};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/students",           post(enroll).get(list))
        .route("/students/:id",       get(get_student))
        .route("/students/:id/goals", put(update_goals))
}

async fn enroll(
    State(state): State<AppState>,
    Json(req): Json<EnrollRequest>,
) -> ServerResult<(StatusCode, Json<Value>)> {
    let student = db::students::create(&state.db, &req).await?;
    let events = vec![AcademyEvent::StudentEnrolled {
        student_id: student.id,
        username: student.username.clone(),
        email: student.email.clone(),
        initial_biome_slug: req.topic.as_deref().unwrap_or("algorithms-mountains").to_owned(),
        timestamp: Utc::now(),
    }];
    apply_events(&state, &events).await?;
    Ok((StatusCode::CREATED, Json(json!({
        "student_id":      student.id,
        "message":         format!("Welcome to ForgeFabrik Academy, {}!", student.username),
        "initial_biome":   req.topic.as_deref().unwrap_or("algorithms-mountains"),
        "mentor_assigned": "claude-mentor",
        "xp":              student.xp,
        "level":           student.level,
        "enrolled_at":     student.enrolled_at,
    }))))
}

async fn get_student(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<Value>> {
    let student = db::students::get(&state.db, id).await?;
    let quests  = db::quests::list_for_student(&state.db, id).await?;
    let completed   = quests.iter().filter(|q| q.status == QuestStatus::Completed).count();
    let failed      = quests.iter().filter(|q| q.status == QuestStatus::Failed).count();
    let in_progress = quests.iter().filter(|q| q.status == QuestStatus::InProgress).count();
    Ok(Json(json!({ "student": student, "progress": {
        "completed":   completed,
        "failed":      failed,
        "in_progress": in_progress,
    }})))
}

async fn list(State(state): State<AppState>) -> ServerResult<Json<Value>> {
    let students = db::students::list(&state.db).await?;
    let count = students.len();
    Ok(Json(json!({ "students": students, "count": count })))
}

async fn update_goals(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGoalsRequest>,
) -> ServerResult<Json<Value>> {
    if req.goals.is_empty() {
        return Err(crate::error::ServerError::BadRequest("goals must not be empty".into()));
    }
    let student = db::students::update_goals(&state.db, id, req.goals).await?;
    Ok(Json(serde_json::to_value(student)?))
}
