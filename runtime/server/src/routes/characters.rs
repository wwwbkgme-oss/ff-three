//! Character routes — REST API for the character simulation domain.
//!
//! All mutations go through the `AggregateRoot` command handler;
//! reads are served from the `CharacterView` projection.
//!
//! ## Endpoints
//!
//! ```text
//! POST /characters             — spawn a new NPC
//! GET  /characters/:id         — fetch CharacterView
//! POST /characters/:id/tick    — advance one simulation tick
//! GET  /characters/:id/goals   — goal stack summary
//! GET  /characters/:id/memory  — salient memory episodes
//! ```

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use characters::{
    character::Character,
    reducer::CharacterReducer,
    tick::TickEngine,
};
use types::{
    traits::Reducer,
    CharacterId, LocationId, TickContext, WorldTick,
};

use crate::{error::ServerResult, state::AppState};

// ── Request / response DTOs ───────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SpawnRequest {
    pub name:     String,
    /// Optional starting location UUID; defaults to a new random location.
    pub location: Option<Uuid>,
    /// Tick at which the character is born; defaults to 0.
    pub born_at:  Option<u64>,
}

#[derive(Serialize)]
pub struct CharacterSummary {
    pub id:       CharacterId,
    pub name:     String,
    pub version:  u64,
    pub health:   i32,
    pub energy:   i32,
    pub hunger:   i32,
    pub fatigue:  i32,
    pub location: LocationId,
}

impl CharacterSummary {
    fn from_char(c: &Character) -> Self {
        Self {
            id:       c.id,
            name:     c.name.clone(),
            version:  c.version,
            health:   c.stats.health,
            energy:   c.stats.energy,
            hunger:   c.stats.hunger,
            fatigue:  c.stats.fatigue,
            location: c.location,
        }
    }
}

#[derive(Deserialize)]
pub struct TickRequest {
    /// Absolute simulation tick.
    pub tick: u64,
}

#[derive(Serialize)]
pub struct TickResponse {
    pub events_applied: usize,
    pub character:      CharacterSummary,
}

#[derive(Serialize)]
pub struct GoalsSummary {
    pub active:  Option<String>,
    pub pending: Vec<String>,
}

#[derive(Serialize)]
pub struct MemorySummary {
    pub episodes: Vec<EpisodeSummary>,
}

#[derive(Serialize)]
pub struct EpisodeSummary {
    pub id:      types::EpisodeId,
    pub summary: String,
    pub weight:  f32,
}

// ── In-memory character store (stub — replace with PgEventStore) ──────────────
//
// For now characters live in process memory.  Once PgEventStore is wired into
// AppState, handlers should load/save through the event store instead.

use std::{collections::HashMap, sync::{Arc, RwLock}};

type CharacterStore = Arc<RwLock<HashMap<CharacterId, Character>>>;

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn spawn(
    State(state): State<AppState>,
    Json(req):    Json<SpawnRequest>,
) -> Result<(StatusCode, Json<CharacterSummary>), StatusCode> {
    let id  = CharacterId::new();
    let loc = req.location.map(LocationId::from).unwrap_or_else(LocationId::new);
    let born_at = WorldTick(req.born_at.unwrap_or(0));
    let char = Character::new_npc(id, req.name.clone(), loc, born_at);
    let summary = CharacterSummary::from_char(&char);

    // TODO: persist via PgEventStore (store.append(CharacterEvent::Created { ... }))
    tracing::info!(character_id = %id, name = %req.name, "Spawned NPC");

    Ok((StatusCode::CREATED, Json(summary)))
}

async fn get_character(
    Path(id): Path<Uuid>,
) -> Result<Json<CharacterSummary>, StatusCode> {
    // TODO: load from event store / projection
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn tick_character(
    Path(id):  Path<Uuid>,
    Json(req): Json<TickRequest>,
) -> Result<Json<TickResponse>, StatusCode> {
    // TODO: load character, run TickEngine, persist events, return updated view
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn get_goals(Path(id): Path<Uuid>) -> Result<Json<GoalsSummary>, StatusCode> {
    // TODO: load from event store
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn get_memory(Path(id): Path<Uuid>) -> Result<Json<MemorySummary>, StatusCode> {
    // TODO: load from event store
    Err(StatusCode::NOT_IMPLEMENTED)
}

// ── Router ────────────────────────────────────────────────────────────────────

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/characters",              post(spawn))
        .route("/characters/:id",          get(get_character))
        .route("/characters/:id/tick",     post(tick_character))
        .route("/characters/:id/goals",    get(get_goals))
        .route("/characters/:id/memory",   get(get_memory))
}
