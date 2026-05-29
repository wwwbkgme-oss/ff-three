//! Character routes — REST API for the character simulation domain.
//!
//! ## Endpoints
//! ```text
//! POST /characters             — spawn a new NPC
//! GET  /characters/:id         — fetch CharacterView
//! POST /characters/:id/tick    — advance one simulation tick
//! GET  /characters/:id/goals   — goal stack summary
//! GET  /characters/:id/memory  — salient memory episodes
//! ```
//!
//! All mutations flow through `AggregateRoot::handle` → `EventStore::append`
//! → `CharacterReducer::apply`. The handler never mutates Character state
//! directly — only via the event pipeline.

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
    tick::TickEngine,
};
use events::{CharacterEvent, CharacterKind as EvtKind, EventStore, ExpectedVersion, StreamId};
use types::{
    traits::{AggregateRoot, CommandContext, Reducer},
    CharacterId, LocationId, TickContext, WorldTick,
};
use characters::reducer::CharacterReducer;

use crate::{error::ServerResult, event_store::PgEventStore, state::AppState};

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SpawnRequest {
    pub name:     String,
    pub location: Option<Uuid>,
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
    pub tick: u64,
}

#[derive(Serialize)]
pub struct TickResponse {
    pub events_applied: usize,
    pub character:      CharacterSummary,
    pub new_version:    u64,
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

// ── Shared helper: load + replay Character from event store ──────────────────

async fn load_character(store: &PgEventStore, id: Uuid) -> Result<Character, StatusCode> {
    let stream  = StreamId::from_uuid(id);
    let stored  = store.load_stream(stream).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if stored.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let char_id   = CharacterId::from(id);
    let initial   = Character::new_npc(char_id, "", LocationId::new(), WorldTick(0));
    let character = stored.iter().try_fold(initial, |state, ev| {
        let event: CharacterEvent = ev.deserialize()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok::<_, StatusCode>(CharacterReducer::apply(state, &event))
    })?;

    Ok(character)
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /characters — Neuen NPC in der Academy spawnen.
async fn spawn(
    State(state): State<AppState>,
    Json(req):    Json<SpawnRequest>,
) -> Result<(StatusCode, Json<CharacterSummary>), StatusCode> {
    let id      = CharacterId::new();
    let loc     = req.location.map(LocationId::from).unwrap_or_else(LocationId::new);
    let born_at = WorldTick(req.born_at.unwrap_or(0));
    let char    = Character::new_npc(id, req.name.clone(), loc, born_at);

    // Emittiere CharacterEvent::Created und persistiere es
    let event = CharacterEvent::Created {
        id,
        kind:     EvtKind::Npc,
        name:     req.name.clone(),
        location: loc,
        born_at,
    };
    let payload = serde_json::to_value(&event)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let store   = PgEventStore::new(state.db.clone());
    let stream  = StreamId::from_uuid(id.inner());
    store.append(stream, ExpectedVersion::NoStream, vec![payload]).await
        .map_err(|e| {
            tracing::error!(error = %e, "spawn: event store error");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(character_id = %id, name = %req.name, "NPC spawned");
    let summary = CharacterSummary::from_char(&char);
    Ok((StatusCode::CREATED, Json(summary)))
}

/// GET /characters/:id — CharacterView aus Event-Store laden (Replay).
async fn get_character(
    State(state): State<AppState>,
    Path(id):     Path<Uuid>,
) -> Result<Json<CharacterSummary>, StatusCode> {
    let store = PgEventStore::new(state.db.clone());
    let char  = load_character(&store, id).await?;
    Ok(Json(CharacterSummary::from_char(&char)))
}

/// POST /characters/:id/tick — einen Simulations-Tick ausführen.
async fn tick_character(
    State(state): State<AppState>,
    Path(id):     Path<Uuid>,
    Json(req):    Json<TickRequest>,
) -> Result<Json<TickResponse>, StatusCode> {
    let store = PgEventStore::new(state.db.clone());
    let char  = load_character(&store, id).await?;

    // Deterministischen TickContext erstellen (keine Wall-Clock-Zeit)
    let realm   = uuid::Uuid::new_v4(); // pro Tick frisch — Arena-Isolation
    let ctx     = TickContext::new(req.tick, realm, 1);
    let events  = TickEngine::tick(&char, &ctx);

    if events.is_empty() {
        // Nichts zu persistieren — Character unverändert zurückgeben
        return Ok(Json(TickResponse {
            events_applied: 0,
            character:      CharacterSummary::from_char(&char),
            new_version:    char.version,
        }));
    }

    // Events serialisieren + persistieren
    let payloads: Result<Vec<_>, _> = events.iter()
        .map(|e| serde_json::to_value(e))
        .collect();
    let payloads = payloads.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stream = StreamId::from_uuid(id);
    let new_version = store
        .append(stream, ExpectedVersion::Exact(char.version), payloads)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "tick: event store error");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Neuen Zustand aus Events projizieren
    let updated = events.iter()
        .fold(char, |s, e| CharacterReducer::apply(s, e));

    tracing::debug!(character_id = %id, tick = req.tick, events = events.len(), "tick applied");
    Ok(Json(TickResponse {
        events_applied: events.len(),
        character:      CharacterSummary::from_char(&updated),
        new_version,
    }))
}

/// GET /characters/:id/goals — Goal-Stack des Characters.
async fn get_goals(
    State(state): State<AppState>,
    Path(id):     Path<Uuid>,
) -> Result<Json<GoalsSummary>, StatusCode> {
    let store = PgEventStore::new(state.db.clone());
    let char  = load_character(&store, id).await?;

    Ok(Json(GoalsSummary {
        active:  char.goals.active.as_ref().map(|g| format!("{:?}", g.kind)),
        pending: char.goals.pending.iter()
            .map(|g| format!("{:?}", g.kind))
            .collect(),
    }))
}

/// GET /characters/:id/memory — Wichtigste Memory-Episoden.
async fn get_memory(
    State(state): State<AppState>,
    Path(id):     Path<Uuid>,
) -> Result<Json<MemorySummary>, StatusCode> {
    let store = PgEventStore::new(state.db.clone());
    let char  = load_character(&store, id).await?;

    let episodes = char.memory.episodes.iter()
        .map(|ep| EpisodeSummary {
            id:      ep.id,
            summary: ep.summary.clone(),
            weight:  ep.weight,
        })
        .collect();

    Ok(Json(MemorySummary { episodes }))
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
