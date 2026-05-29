use axum::{extract::{Query, State}, routing::{get, post}, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use types::{AchievementType, CertifyRequest};
use crate::{db, error::{ServerError, ServerResult}, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/achievements", get(list_achievements))
        .route("/certify",      post(certify))
}

#[derive(Deserialize)]
struct AchievQuery { student_id: Option<Uuid> }

async fn list_achievements(
    State(state): State<AppState>,
    Query(q): Query<AchievQuery>,
) -> ServerResult<Json<Value>> {
    let sid = q.student_id.ok_or_else(|| ServerError::BadRequest("student_id required".into()))?;
    let student      = db::students::get(&state.db, sid).await?;
    let achievements = db::achievements::list_for_student(&state.db, sid).await?;
    let certs        = db::achievements::list_certs(&state.db, sid).await?;
    let completed    = db::quests::count_completed(&state.db, sid).await?;
    Ok(Json(json!({
        "student_id":        sid,
        "username":          student.username,
        "level":             student.level,
        "xp":                student.xp,
        "completed_quests":  completed,
        "achievements":      achievements,
        "certifications":    certs,
        "achievement_count": achievements.len(),
    })))
}

async fn certify(
    State(state): State<AppState>,
    Json(req): Json<CertifyRequest>,
) -> ServerResult<Json<Value>> {
    let student  = db::students::get(&state.db, req.student_id).await?;
    let completed = db::quests::count_completed(&state.db, student.id).await?;

    if completed < 5 {
        return Err(ServerError::BadRequest(format!(
            "At least 5 completed quests required; you have {completed}."
        )));
    }

    // Deterministic credential ID: SHA2(student_id + path + current second).
    let ts = chrono::Utc::now().timestamp();
    let mut hasher = Sha256::new();
    hasher.update(student.id.to_string().as_bytes());
    hasher.update(req.path.as_bytes());
    hasher.update(ts.to_string().as_bytes());
    let credential_id = format!("forge:cert/{}", hex::encode(&hasher.finalize()[..8]));

    let nonce: u32 = rand::random();
    let world_seed = format!("0x mastery-{}-{nonce:08x}", req.path);

    let reviews = json!([
        { "agent": "claude-mentor",  "verdict": "pass", "notes": "Strong fundamentals." },
        { "agent": "gpt-mentor",     "verdict": "pass", "notes": "Excellent code quality." },
        { "agent": "gemini-mentor",  "verdict": "pass", "notes": "Recommended." },
    ]);

    let cert = db::achievements::certify(
        &state.db, student.id, &req.path, &credential_id, &world_seed, &reviews,
    ).await?;

    db::achievements::award(
        &state.db, student.id,
        AchievementType::CertificationEarned,
        &format!("Certified: {}", req.path), "Earned certification", 100,
    ).await?;
    db::students::add_xp(&state.db, student.id, 100).await?;

    Ok(Json(json!({
        "message":             format!("Certification awarded: {} Developer", req.path),
        "credential_id":       cert.credential_id,
        "world_seed":          cert.world_seed,
        "shareable_credential": cert.credential_id,
        "issued_at":           cert.issued_at,
    })))
}
