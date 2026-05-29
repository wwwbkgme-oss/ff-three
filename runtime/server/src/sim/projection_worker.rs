//! Projection-Catch-Up-Worker — P2.3 aus NEXT.md.
//!
//! Pollt periodisch den globalen Event-Log (`load_since(checkpoint)`) und
//! aktualisiert die `CharacterView`-Read-Model-Tabelle.
//!
//! ## Warum Projection?
//! Der primäre Store enthält Events (write model). `GET /characters/:id` und
//! ähnliche Reads würden bei jedem Request alle Events replapen → teuer.
//! Die Projection hält eine materialisierte, aktuelle Sicht bereit.
//!
//! ## Tabellen-Schema (migration 006 — muss separat ausgeführt werden)
//! ```sql
//! CREATE TABLE IF NOT EXISTS character_views (
//!     id         UUID PRIMARY KEY,
//!     data       JSONB NOT NULL,
//!     checkpoint BIGINT NOT NULL DEFAULT 0,
//!     updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
//! );
//! CREATE TABLE IF NOT EXISTS projection_checkpoints (
//!     name       TEXT PRIMARY KEY,
//!     offset     BIGINT NOT NULL DEFAULT 0
//! );
//! ```

use std::time::Duration;

use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use characters::{character::Character, reducer::CharacterReducer};
use events::{CharacterEvent, StreamId};
use projections::CharacterView;
use types::{CharacterId, LocationId, WorldTick, traits::Reducer};

use crate::{event_store::PgEventStore, state::AppState};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const BATCH_SIZE:    usize    = 100;
const CHECKPOINT_NAME: &str   = "character_view";

// ── Öffentliche API ───────────────────────────────────────────────────────────

pub fn spawn(state: AppState) -> JoinHandle<()> {
    tokio::spawn(run(state))
}

// ── Worker-Loop ───────────────────────────────────────────────────────────────

async fn run(state: AppState) {
    info!("projection_worker: gestartet ({}ms Intervall)", POLL_INTERVAL.as_millis());

    loop {
        if let Err(e) = catch_up(&state).await {
            error!(error = %e, "projection_worker: catch-up fehlgeschlagen");
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn catch_up(state: &AppState) -> anyhow::Result<()> {
    let store      = PgEventStore::new(state.db.clone());
    let checkpoint = load_checkpoint(state).await?;
    let events     = store.load_since(checkpoint, BATCH_SIZE).await
        .map_err(|e| anyhow::anyhow!("load_since: {e}"))?;

    if events.is_empty() { return Ok(()); }

    let last_offset = events.last().map(|e| e.global_offset).unwrap_or(checkpoint);
    let mut count   = 0;

    for stored_event in &events {
        let event: CharacterEvent = match stored_event.deserialize() {
            Ok(e)  => e,
            Err(e) => {
                warn!(offset = stored_event.global_offset, error = %e, "projection_worker: skip unbekanntes Event");
                continue;
            }
        };

        let npc_id = stored_event.stream_id.0;
        if let Err(e) = update_character_view(state, npc_id, &event, stored_event.global_offset).await {
            warn!(npc_id = %npc_id, error = %e, "projection_worker: view-Update fehlgeschlagen");
        } else {
            count += 1;
        }
    }

    if count > 0 {
        debug!(events = count, new_checkpoint = last_offset, "projection_worker: Batch verarbeitet");
        save_checkpoint(state, last_offset + 1).await?;
    }
    Ok(())
}

async fn update_character_view(
    state:      &AppState,
    npc_id:     uuid::Uuid,
    new_event:  &CharacterEvent,
    checkpoint: u64,
) -> anyhow::Result<()> {
    // Aktuelle View laden (oder neu erstellen)
    let char_id = CharacterId::from(npc_id);
    let current = load_view(state, npc_id).await?;

    let base_char = current.as_ref()
        .map(|v| character_from_view(v, char_id))
        .unwrap_or_else(|| Character::new_npc(char_id, "", LocationId::new(), WorldTick(0)));

    let updated_char = CharacterReducer::apply(base_char, new_event);
    let view         = CharacterView::from_character(&updated_char, WorldTick(0), checkpoint);

    persist_view(state, npc_id, &view, checkpoint).await
}

/// Minimale Character-Rekonstruktion aus einer CharacterView (für inkrementelles Apply).
fn character_from_view(view: &CharacterView, id: CharacterId) -> Character {
    let mut c = Character::new_npc(id, &view.name, view.location, WorldTick(0));
    c.version          = view.version;
    c.stats.health     = view.health;
    c.stats.energy     = view.energy;
    c.stats.hunger     = view.hunger;
    c.stats.fatigue    = view.fatigue;
    c.stats.social_need = view.social_need;
    c
}

// ── Datenbank-Helfer ──────────────────────────────────────────────────────────

async fn load_checkpoint(state: &AppState) -> anyhow::Result<u64> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT offset FROM projection_checkpoints WHERE name = $1"
    )
    .bind(CHECKPOINT_NAME)
    .fetch_optional(&state.db)
    .await?;

    Ok(row.map(|(o,)| o as u64).unwrap_or(0))
}

async fn save_checkpoint(state: &AppState, offset: u64) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO projection_checkpoints (name, offset) VALUES ($1, $2)
         ON CONFLICT (name) DO UPDATE SET offset = $2"
    )
    .bind(CHECKPOINT_NAME)
    .bind(offset as i64)
    .execute(&state.db)
    .await?;
    Ok(())
}

async fn load_view(state: &AppState, id: uuid::Uuid) -> anyhow::Result<Option<CharacterView>> {
    let row: Option<(serde_json::Value,)> = sqlx::query_as(
        "SELECT data FROM character_views WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?;

    match row {
        None       => Ok(None),
        Some((v,)) => Ok(Some(serde_json::from_value(v)?)),
    }
}

async fn persist_view(
    state:      &AppState,
    id:         uuid::Uuid,
    view:       &CharacterView,
    checkpoint: u64,
) -> anyhow::Result<()> {
    let data = serde_json::to_value(view)?;
    sqlx::query(
        "INSERT INTO character_views (id, data, checkpoint)
         VALUES ($1, $2, $3)
         ON CONFLICT (id) DO UPDATE
         SET data = $2, checkpoint = $3, updated_at = NOW()"
    )
    .bind(id)
    .bind(data)
    .bind(checkpoint as i64)
    .execute(&state.db)
    .await?;
    Ok(())
}
