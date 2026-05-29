//! Route assembly and shared event-application helper.
//!
//! BKG runtime rule: orchestriert – no business truth lives here.
//! Domain logic is in domain/; handlers just wire I/O around it.

pub mod achievements;
pub mod biomes;
pub mod characters;
pub mod curriculum;
pub mod groups;
pub mod quests;
pub mod sandbox;
pub mod students;

use axum::{Router, http::{HeaderName, Method}, routing::get, response::Json};
use serde_json::json;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

use events::AcademyEvent;

use crate::{db, error::ServerResult, state::AppState};

/// Assemble the complete application router with middleware.
pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            HeaderName::from_static("content-type"),
            HeaderName::from_static("authorization"),
        ]);

    Router::new()
        .route("/health",    get(health))
        .route("/readiness", get(|| async { Json(json!({ "status": "ready" })) }))
        .merge(students::router())
        .merge(curriculum::router())
        .merge(biomes::router())
        .merge(quests::router())
        .merge(sandbox::router())
        .merge(groups::router())
        .merge(achievements::router())
        .merge(characters::router())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "service": "forge-fabrik-academy" }))
}

// ── Shared event applier ──────────────────────────────────────────────────────

/// Persist the side-effects of domain events into the database.
/// This is the Single Mutation Path: all DB writes from domain events go here.
pub(crate) async fn apply_events(
    state: &AppState,
    events: &[AcademyEvent],
) -> ServerResult<()> {
    for event in events {
        match event {
            AcademyEvent::QuestCompleted { student_id, quest_id, .. } => {
                db::quests::complete_for_student(&state.db, *student_id, *quest_id, true).await?;
            }
            AcademyEvent::QuestFailed { student_id, quest_id, .. } => {
                db::quests::complete_for_student(&state.db, *student_id, *quest_id, false).await?;
            }
            AcademyEvent::XpGained { student_id, new_total, .. } => {
                // set_xp_level derives the level from new_total (deterministic).
                db::students::set_xp_level(&state.db, *student_id, *new_total).await?;
            }
            AcademyEvent::LevelUp { .. } => {
                // Already handled implicitly by set_xp_level above.
            }
            AcademyEvent::AchievementEarned { student_id, achievement_type, title, xp_reward, .. } => {
                db::achievements::award(
                    &state.db, *student_id, achievement_type.clone(),
                    title, &format!("Earned: {title}"), *xp_reward,
                ).await?;
            }
            AcademyEvent::BiomeStateChanged { biome_id, new_state, .. } => {
                db::biomes::set_state(&state.db, *biome_id, new_state.clone()).await?;
            }
            AcademyEvent::ConceptMasteryUpdated { student_id, concept, mastery, .. } => {
                // Load current knowledge map, update in memory, persist.
                let student = db::students::get(&state.db, *student_id).await?;
                let mut kg = types::KnowledgeGraph::from_json(*student_id, &student.knowledge_map);
                kg.update_mastery(concept, *mastery);
                db::students::update_knowledge_map(&state.db, *student_id, &kg.to_json()).await?;
            }
            _ => {} // informational events not requiring DB writes
        }
    }
    Ok(())
}
