use axum::{extract::{Path, State}, routing::{get, post}, Json, Router};
use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use events::AcademyEvent;
use types::{BiomeSummary, ExploreRequest};

use crate::{db, error::ServerResult, routes::apply_events, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/biomes",     get(list))
        .route("/biomes/:id", get(get_biome))
        .route("/explore",    post(explore))
}

async fn list(State(state): State<AppState>) -> ServerResult<Json<Value>> {
    let biomes = db::biomes::list(&state.db).await?;
    let summaries: Vec<BiomeSummary> = biomes.into_iter().map(|b| BiomeSummary {
        id: b.id, name: b.name, slug: b.slug, domain: b.domain,
        state: b.state, active_students: b.active_students, available_quests: 0,
    }).collect();
    let count = summaries.len();
    Ok(Json(json!({ "biomes": summaries, "count": count })))
}

async fn get_biome(
    State(state): State<AppState>, Path(id): Path<Uuid>,
) -> ServerResult<Json<Value>> {
    let biome  = db::biomes::get(&state.db, id).await?;
    let quests = db::quests::list_by_biome(&state.db, id).await?;
    Ok(Json(json!({ "biome": biome, "quests": quests, "quest_count": quests.len() })))
}

async fn explore(
    State(state): State<AppState>, Json(req): Json<ExploreRequest>,
) -> ServerResult<Json<Value>> {
    let biome   = db::biomes::get_by_slug(&state.db, &req.biome_slug).await?;
    let student = db::students::get(&state.db, req.student_id).await?;
    let quests  = db::quests::list_by_biome(&state.db, biome.id).await?;

    let events = vec![AcademyEvent::BiomeEntered {
        student_id: student.id, biome_id: biome.id,
        biome_slug: biome.slug.clone(), timestamp: Utc::now(),
    }];
    apply_events(&state, &events).await?;
    db::students::set_biome(&state.db, student.id, biome.id).await?;
    db::biomes::incr_active(&state.db, biome.id).await?;

    let titles: Vec<String> = quests.iter().map(|q| q.title.clone()).collect();
    Ok(Json(json!({
        "biome":             biome,
        "message":           format!("Entering {}…", biome.name),
        "available_quests":  titles,
        "security_status":   "Sandbox active",
    })))
}
