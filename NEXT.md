# NEXT — Planned Work

Ordered by urgency.  ✅ = done on this branch.

---

## P0 — Critical path  ✅ All done

| Item | Commit |
|---|---|
| EventStore trait + InMemoryEventStore | `a6b5ec4` |
| Character.version (optimistic concurrency) | `94ec9b0` |
| Reducer increments version | `c453afa` |
| Wire LLM driver into AppState | `af74565` |
| PgEventStore + migration 005 | `1a53d6a` |

---

## P1 — Important  ✅ All done

| Item | Commit |
|---|---|
| TickContext + DeterministicRng | `dde74ce` |
| Planner + TickEngine use TickContext | `f87dd42` |
| runtime/projections + CharacterView | `da0fe54` |
| Character REST API skeleton | `18e0990` |
| Move LLM to runtime/drivers (arch refactor) | `8f5602d` |
| ARCHITECTURE.md + PLUGIN_VS_DRIVER.md | `ba3b2dc` |

---

## P2 — Next sprint

### ~~P2.1 · Fill character REST handlers~~ ✅ `9641d15`

All five routes wired through `PgEventStore`:
- `spawn`: `CharacterEvent::Created` → `append(NoStream)`
- `get_character`: `load_stream` → `CharacterReducer::replay` → `CharacterSummary`
- `tick_character`: `TickEngine::tick` → `append(Exact(version))` → projizierter Zustand
- `get_goals`: replay → GoalStack summary
- `get_memory`: replay → salient episodes

OCC-Guard aktiv: `ExpectedVersion::Exact(char.version)` verhindert concurrent writes.

---

### P2.1 (archived reference)

```rust
// POST /characters
let evt = CharacterEvent::Created { id, kind: CharacterKind::Npc, name, location, born_at };
let payload = serde_json::to_value(&evt)?;
let ver = store.append(StreamId::from_uuid(id.inner()),
                       ExpectedVersion::NoStream, vec![payload]).await?;
// also emit CharacterCreated to projection bus

// GET /characters/:id
let stored = store.load_stream(StreamId::from_uuid(id)).await?;
let char = Character::replay(Character::default_npc(id), &deserialise(stored));
Json(CharacterView::from_character(&char, current_tick, stored.last_offset()))
```

---

<<<<<<< HEAD
### ~~P2.2 · World simulation loop~~ ✅ `sim/tick_worker.rs`

1 Hz Tokio-Hintergrundtask: TickEngine::tick → PgEventStore::append.
TickContext deterministisch aus (tick, realm_seed). OCC-Guard aktiv.
=======
### ~~P2.2 · World simulation loop~~ ✅ `sim/tick_worker.rs` + in `main.rs` gestartet

1 Hz Tokio-Task: TickEngine::tick → PgEventStore::append. OCC-Guard aktiv.
>>>>>>> 4fed4b8b73a6745da2bba63965f3bb6241f6f4fc

---

### P2.2 (archived reference)

```rust
loop {
    let tick = WorldTick(counter.fetch_add(1, SeqCst));
    let ctx  = TickContext { tick, realm, rng_seed: tick.0 ^ realm_seed, delta_ticks: 1 };
    for npc in world.active_npcs() {
        let events = TickEngine::tick(&npc, &ctx);
        store.append(
            StreamId::from_uuid(npc.id.inner()),
            ExpectedVersion::Exact(npc.version),
            serialise(events),
        ).await?;
    }
    tokio::time::sleep(TICK_INTERVAL).await;
}
```

---

<<<<<<< HEAD
### ~~P2.3 · Projection catch-up worker~~ ✅ `sim/projection_worker.rs`

500ms Polling, load_since(checkpoint) → character_views UPSERT.
Migration 006: character_views + projection_checkpoints Tabellen.
=======
### ~~P2.3 · Projection catch-up worker~~ ✅ `sim/projection_worker.rs` + Migration 006

500 ms Polling, load_since(checkpoint) → character_views UPSERT.
>>>>>>> 4fed4b8b73a6745da2bba63965f3bb6241f6f4fc

---

### P2.3 (archived reference)
Polls `store.load_since(checkpoint)`, applies to `CharacterView`, upserts
into a `character_views` table, advances checkpoint.

```sql
CREATE TABLE character_views (
    id         UUID PRIMARY KEY,
    data       JSONB NOT NULL,
    checkpoint BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

---

### P2.4 · Multi-realm support

All aggregates carry `RealmId`.  Tick loops, event streams, and projections
are partitioned by realm.  Students join a realm; their NPC lives there.

---

### P2.5 · Snapshot + full replay

- Implement `DeterministicHash for Character` (serde_json → SHA-256)
- Persist `WorldSnapshot` every N events (configurable)
- Load aggregate as `(snapshot, events_since)` instead of full replay
- Cross-node `state_hash` comparison for distributed verification

---

### P2.6 · Social engine

- `ConversationSystem` — schedule-aware NPC conversation matching
- `FactionSystem` — faction membership transitions + world effects
- `ReputationProjection` — per-character public reputation read model

---

### P2.7 · LLM-augmented NPC goals

The `AgentStrategy` trait is already in `foundation/types`.  Wire it:

1. `domain/agents::Orchestrator` builds a prompt from character state
2. `drivers::FreeClient::chat()` → free LLM inference
3. Response parsed into `CharacterCommand::AssignGoal { … }`
4. `AggregateRoot::handle` validates — domain rules still guard correctness

The LLM only *suggests*.  The domain always decides.

---

### P2.8 · Plugin host

For future domain-behaviour plugins (`plugins/`):

```
runtime/plugin_host/
  src/
    lib.rs          — PluginHost struct
    manifest.rs     — Plugin metadata (id, kind, api_version)
    loader.rs       — cdylib dynamic loading via `libloading`
```

Plugin manifest:
```toml
[plugin]
id          = "forgefabrik.npc-economy"
kind        = "domain-behaviour"
api_version = 1
```

First plugin candidate: `plugins/npc-economy` — adds economy goal types.

---

## Invariants to preserve forever

| Invariant | Enforced by |
|---|---|
| No `Utc::now()` in `domain/` or `foundation/` | Layer boundary |
| No `thread_rng()` in `domain/` | Use `DeterministicRng` |
| No DB/network in `domain/` | Layer boundary |
| `drivers` never imported by `domain/` or `foundation/` | `cargo check` + CI |
| `plugins/` = pure domain behaviour only | `docs/PLUGIN_VS_DRIVER.md` |
| `replay(s, events) == seq_apply(s, events)` | `aggregate_replay_matches_sequential_application` test |
| GoalId deterministic (UUIDv5) | `planner_same_input_same_output` test |
| Stats never exceed max | `Stats::clamp()` in reducer |
| `character.version` == events applied | `reducer_version_increments` test |
