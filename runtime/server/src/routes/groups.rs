use axum::{extract::{Path, State}, http::StatusCode, routing::{get, post}, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use types::CreateGroupRequest;
use crate::{db, error::ServerResult, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/groups",              get(list).post(create))
        .route("/groups/:id/join",     post(join))
        .route("/groups/:id/progress", get(progress))
}

async fn list(State(state): State<AppState>) -> ServerResult<Json<Value>> {
    let groups = db::groups::list(&state.db).await?;
    let count = groups.len();
    Ok(Json(json!({ "groups": groups, "count": count })))
}

async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupRequest>,
) -> ServerResult<(StatusCode, Json<Value>)> {
    let group = db::groups::create(&state.db, &req).await?;
    Ok((StatusCode::CREATED, Json(json!({
        "group":   group,
        "message": format!("Group '{}' created. Share the ID to invite peers.", group.id),
    }))))
}

#[derive(Deserialize)]
struct JoinBody { student_id: Uuid }

async fn join(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(body): Json<JoinBody>,
) -> ServerResult<Json<Value>> {
    let member = db::groups::join(&state.db, group_id, body.student_id).await?;
    let group  = db::groups::get(&state.db, group_id).await?;
    Ok(Json(json!({
        "message":    format!("Joined '{}'", group.name),
        "group_id":   group_id,
        "student_id": body.student_id,
        "role":       member.role,
        "joined_at":  member.joined_at,
    })))
}

async fn progress(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> ServerResult<Json<Value>> {
    let p = db::groups::progress(&state.db, id).await?;
    Ok(Json(serde_json::to_value(p)?))
}
