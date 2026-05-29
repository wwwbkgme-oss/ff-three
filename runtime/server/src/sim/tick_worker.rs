//! World-Simulation-Loop — P2.2 aus NEXT.md.
//!
//! Spawnt einen Tokio-Hintergrundtask, der in konfigurierbaren Intervallen
//! `TickEngine::tick()` für alle aktiven NPCs ausführt und die Events
//! in den `PgEventStore` schreibt.
//!
//! ## Determinismus
//! `TickContext` wird aus `(tick, realm, rng_seed)` deterministisch konstruiert
//! — kein `Utc::now()` oder `thread_rng()` in der Domain-Schicht.
//!
//! ## Fehlerbehandlung
//! Optimistic-Concurrency-Konflikte (2 gleichzeitige Writer für dasselbe
//! Aggregate) werden geloggt und übersprungen — nächster Tick versucht es erneut.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use characters::{reducer::CharacterReducer, tick::TickEngine};
use events::{CharacterEvent, EventStore, ExpectedVersion, StreamId};
use types::{
    traits::Reducer,
    CharacterId, LocationId, RealmId, TickContext, WorldTick,
};

use crate::{event_store::PgEventStore, state::AppState};

// ── Konfiguration ─────────────────────────────────────────────────────────────

/// Simulationsfrequenz — 1 Hz = 1 Tick pro Sekunde.
const TICK_INTERVAL: Duration = Duration::from_millis(1000);

/// Maximale NPCs die pro Tick verarbeitet werden (verhindert Überlastung).
const MAX_NPCS_PER_TICK: usize = 50;

// ── Öffentliche API ───────────────────────────────────────────────────────────

/// Spawnt den Tick-Worker als Hintergrundtask.
///
/// Gibt den globalen Tick-Zähler zurück — kann von Handlers gelesen werden.
pub fn spawn(state: AppState, realm: RealmId) -> (JoinHandle<()>, Arc<AtomicU64>) {
    let counter = Arc::new(AtomicU64::new(0));
    let c2      = Arc::clone(&counter);
    let handle  = tokio::spawn(run(state, realm, counter));
    (handle, c2)
}

// ── Interner Loop ─────────────────────────────────────────────────────────────

async fn run(state: AppState, realm: RealmId, counter: Arc<AtomicU64>) {
    info!(?realm, "tick_worker: gestartet ({}ms Intervall)", TICK_INTERVAL.as_millis());

    // Seed aus Realm-ID ableiten (deterministisch)
    let realm_seed = u64::from_le_bytes(realm.as_bytes()[..8].try_into().unwrap_or([0u8; 8]));

    loop {
        let tick_num = counter.fetch_add(1, Ordering::SeqCst);
        let tick     = WorldTick(tick_num);
        let ctx      = TickContext::new(tick_num, realm, 1);

        process_tick(&state, tick, &ctx).await;

        tokio::time::sleep(TICK_INTERVAL).await;
    }
}

async fn process_tick(state: &AppState, tick: WorldTick, ctx: &TickContext) {
    let store = PgEventStore::new(state.db.clone());

    // Aktive NPC-IDs laden (aus den bekannten Streams im event_store).
    // Vereinfachung: wir laden die letzten Streams als Proxy für "aktive NPCs".
    let npc_ids = match load_active_npc_ids(&store).await {
        Ok(ids) => ids,
        Err(e) => {
            error!(error = %e, tick = tick.0, "tick_worker: NPC-Liste laden fehlgeschlagen");
            return;
        }
    };

    debug!(tick = tick.0, npcs = npc_ids.len(), "tick_worker: verarbeite tick");

    let mut processed = 0;
    for npc_id in npc_ids.into_iter().take(MAX_NPCS_PER_TICK) {
        if let Err(e) = tick_npc(&store, npc_id, ctx).await {
            warn!(npc_id = %npc_id, tick = tick.0, error = %e, "tick_worker: NPC-Tick fehlgeschlagen");
        } else {
            processed += 1;
        }
    }

    debug!(tick = tick.0, processed, "tick_worker: tick abgeschlossen");
}

async fn tick_npc(
    store:  &PgEventStore,
    npc_id: uuid::Uuid,
    ctx:    &TickContext,
) -> anyhow::Result<()> {
    use characters::character::Character;

    let stream = StreamId::from_uuid(npc_id);

    // Character aus Event-Store laden (Replay)
    let stored = store.load_stream(stream).await
        .map_err(|e| anyhow::anyhow!("load_stream: {e}"))?;

    if stored.is_empty() { return Ok(()); }

    let char_id  = CharacterId::from(npc_id);
    let loc      = LocationId::new();
    let initial  = Character::new_npc(char_id, "", loc, WorldTick(0));
    let character = stored.iter().try_fold(initial, |state, ev| {
        let event: CharacterEvent = ev.deserialize()?;
        Ok::<_, serde_json::Error>(CharacterReducer::apply(state, &event))
    })?;

    // Tick ausführen (deterministisch, kein I/O)
    let events = TickEngine::tick(&character, ctx);
    if events.is_empty() { return Ok(()); }

    // Events persistieren (OCC-Guard)
    let payloads: Vec<serde_json::Value> = events.iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()?;

    store.append(stream, ExpectedVersion::Exact(character.version), payloads)
        .await
        .map_err(|e| anyhow::anyhow!("append: {e}"))?;

    debug!(npc_id = %npc_id, events = events.len(), tick = ctx.tick.0, "tick_worker: NPC getickt");
    Ok(())
}

/// Einfacher Proxy: lädt Streams aus der DB als "aktive NPCs".
/// In Produktion: eigene `active_characters` Tabelle mit Status.
async fn load_active_npc_ids(store: &PgEventStore) -> anyhow::Result<Vec<uuid::Uuid>> {
    let rows: Vec<(uuid::Uuid,)> = sqlx::query_as(
        "SELECT DISTINCT stream_id FROM event_streams ORDER BY stream_id LIMIT 100"
    )
    .fetch_all(store.pool())
    .await
    .map_err(|e| anyhow::anyhow!("load_active_npcs: {e}"))?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}
