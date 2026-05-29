# NEXT — Planned Work

Ordered by urgency.  ✅ = done on this branch.

---

## P0 — Critical path  ✅ All done

| Item | Status | Commit |
|---|---|---|
| EventStore trait + InMemoryEventStore | ✅ | `a6b5ec4` |
| Character.version (optimistic concurrency) | ✅ | `94ec9b0` |
| Reducer increments version | ✅ | `c453afa` |
| Wire `llm-free` into `runtime/server` | ✅ | `af74565` |
| PgEventStore + migration 005 | ✅ | `1a53d6a` |

---

## P1 — Important soon  ✅ All done

| Item | Status | Commit |
|---|---|---|
| TickContext | ✅ | `dde74ce` |
| DeterministicRng | ✅ | `dde74ce` |
| Planner + TickEngine use TickContext | ✅ | `f87dd42` |
| runtime/projections + CharacterView | ✅ | `da0fe54` |
| Character REST API skeleton | ✅ | `18e0990` |

---

## P2 — Next sprint

### P2.1 · Fill character REST handlers

The five routes exist but return `NOT_IMPLEMENTED`.
Wire them up through `PgEventStore`:

```rust
// POST /characters
let evt = CharacterEvent::Created { id, kind, name, location, born_at };
let payload = serde_json::to_value(&evt)?;
store.append(StreamId::from_uuid(id.inner()), ExpectedVersion::NoStream, vec![payload]).await?;

// GET /characters/:id
let stored = store.load_stream(StreamId::from_uuid(id)).await?;
let char = Character::replay(Character::new_npc(...), &deserialise(stored));
let view = CharacterView::from_character(&char, tick, stored.last_offset());
```

---

### P2.2 · World simulation loop

Background task in `runtime/server` that advances all NPC characters one tick
per interval, persisting events through PgEventStore:

```rust
loop {
    let tick = WorldTick(counter.fetch_add(1, SeqCst));
    let ctx  = TickContext { tick, realm, rng_seed: tick.0 ^ realm_seed, delta_ticks: 1 };
    for npc in world.active_npcs() {
        let events = TickEngine::tick(&npc, &ctx);
        store.append(
            StreamId::from_uuid(npc.id.inner()),
            ExpectedVersion::Exact(npc.version),
            serialise_events(events),
        ).await?;
    }
    tokio::time::sleep(TICK_INTERVAL).await;
}
```

---

### P2.3 · Projection catch-up worker

Background task that calls `store.load_since(checkpoint)`, applies events to
`CharacterView` and upserts into a `character_views` Postgres table:

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

All aggregates get a `RealmId`.  Characters in different realms are fully
isolated — tick loops, event streams, and projections are partitioned by realm.

---

### P2.5 · Snapshot + full replay

- `DeterministicHash for Character` (serde_json serialize → SHA-256)
- Persist `WorldSnapshot` every N events (configurable)
- Load aggregate as `(snapshot, events_since)` instead of full replay
- Cross-node verification: compare `state_hash` across replicas

---

### P2.6 · Social engine

- `ConversationSystem` — schedule-aware NPC conversation matching
- `FactionSystem` — faction membership changes + world effects
- `ReputationProjection` — per-character reputation read model

---

### P2.7 · LLM-augmented goal selection

The `AgentStrategy` trait is in `foundation/types`.  Wire it:

1. `domain/agents::Orchestrator` → builds prompt from character state
2. `FreeClient::chat()` → free-tier LLM inference
3. Response parsed into `CharacterCommand::AssignGoal { ... }`
4. Validated by `AggregateRoot::handle` — domain rules still guard correctness

---

## Invariants to preserve forever

| Invariant | Enforced by |
|---|---|
| No `Utc::now()` in `domain/` | Layer boundary |
| No `thread_rng()` in `domain/` | Use `DeterministicRng` |
| No DB/network in `domain/` | Layer boundary |
| Every state change = one `CharacterEvent` | `CharacterReducer` |
| `replay(s, events) == sequential_apply(s, events)` | `aggregate_replay_matches_sequential_application` test |
| GoalId in planner is deterministic (UUIDv5) | `planner_same_input_same_output` test |
| Stats never exceed declared max | `Stats::clamp()` in reducer |
| `character.version` == events applied | `reducer_version_increments` test |
