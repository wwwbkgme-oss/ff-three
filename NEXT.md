# NEXT — Planned Work

Ordered by urgency.  P0 = blocks other work.  P1 = important soon.  P2 = future.

---

## P0 — Critical path

### P0.1 · EventStore

**Why now:** The character aggregate emits events but has nowhere to persist them.
Without a store, replay is impossible and every restart loses all state.

**What to build — `foundation/events/src/store.rs`:**

```rust
pub struct StreamId(Uuid);          // one stream per aggregate instance
pub enum ExpectedVersion { Any, NoStream, Exact(u64) }
pub struct StoredEvent { stream_id, sequence, global_offset, payload: Value }

#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, stream_id, expected_version, events: Vec<Value>) -> Result<u64>;
    async fn load_stream(&self, stream_id) -> Result<Vec<StoredEvent>>;
    async fn load_since(&self, from_offset, limit) -> Result<Vec<StoredEvent>>;
    async fn stream_version(&self, stream_id) -> Result<u64>;
}
```

Also: `InMemoryEventStore` (Arc<RwLock<>>, for tests) and `PgEventStore` (Postgres, for production).

**Upstream hint:** `StreamId::from_uuid(character.id.inner())` is the natural key for character streams.

---

### P0.2 · Character.version (optimistic concurrency)

**Why now:** Without a version counter on `Character`, concurrent command handlers
cannot detect write-write conflicts before appending to the event store.

**What to build:**

```rust
pub struct Character {
    pub version: u64,   // = events applied since initial state
    // ...
}
```

- `Character::new_npc()` → `version: 0`
- `CharacterReducer::apply()` → `state.version += 1` after every event
- Call-site: `store.append(stream_id, ExpectedVersion::Exact(char.version), events)`

---

### P0.3 · Wire `llm-free` into `runtime/server`

**Why now:** `runtime/server/src/llm.rs` is hardcoded to `api.openai.com`.
The `plugin-llm-free` crate already exists and passes tests — just needs wiring.

**Steps:**
1. Add `llm-free` to `runtime/server/Cargo.toml` deps
2. Replace `LlmClient::new(api_key, model)` with `FreeClient::from_env()` in `AppState`
3. Remove `OPENAI_API_KEY` / `OPENAI_MODEL` from required env vars; mark them optional
4. Update `.env.example` with the 6 free-provider keys

---

## P1 — Important soon

### P1.1 · TickContext

**Why:** `TickEngine::tick(character, WorldTick)` leaks a primitive into the API.
When weather, active events, or multi-realm support arrive, callers would need
to pass a growing list of positional args.  Bundle them now.

```rust
pub struct TickContext {
    pub tick:        WorldTick,
    pub realm:       RealmId,
    pub rng_seed:    u64,      // derived from realm + tick
    pub delta_ticks: u64,      // usually 1; larger during catch-up
}
impl TickContext {
    pub fn test(tick: u64) -> Self { ... }
    pub fn advance(self) -> Self { ... }
}
```

**Diff:** `TickEngine::tick(character, tick)` → `TickEngine::tick(character, ctx)`,
`Planner::suggest(character, tick)` → `Planner::suggest(character, ctx)`.
All 17 tests and 2 examples need the call-sites updated.

---

### P1.2 · DeterministicRng

**Why:** Goal priority jitter, loot rolls, mood noise, NPC "mistakes" all need
randomness — but must be replay-safe.  `rand::thread_rng()` is forbidden in
`domain/`.

```rust
// foundation/types/src/rng.rs
pub struct DeterministicRng { state: u64 }
impl DeterministicRng {
    pub fn new(global_seed: u64, tick: WorldTick, entity_id: u128) -> Self
    pub fn next_u64(&mut self) -> u64      // xorshift64*
    pub fn next_f64(&mut self) -> f64      // [0.0, 1.0)
    pub fn next_range(&mut self, min: i64, max: i64) -> i64
    pub fn shuffle<T>(&mut self, slice: &mut [T])
    pub fn chance(&mut self, p: f64) -> bool
}
```

Seed: `DeterministicRng::new(ctx.rng_seed, ctx.tick, char_id.inner().as_u128())`

---

### P1.3 · Projection layer (`runtime/projections`)

**Why:** The `Character` aggregate carries the full goal stack, memory graph, and
relationship edges — too heavy for API responses and leaderboards.  Projections
maintain lightweight read models rebuilt by replaying the event store.

**Skeleton:**

```
runtime/projections/
  src/
    lib.rs               — Projection trait + checkpoint pattern
    character_view.rs    — CharacterView { id, name, stats, location, version, checkpoint }
```

```rust
pub struct CharacterView {
    pub id:          CharacterId,
    pub name:        String,
    pub health:      i32,
    pub energy:      i32,
    pub hunger:      i32,
    pub location:    LocationId,
    pub version:     u64,
    pub checkpoint:  u64,   // last global_offset applied
}
impl CharacterView {
    pub fn from_character(c: &Character, at: WorldTick, checkpoint: u64) -> Self
    pub fn apply(&mut self, event: &CharacterEvent) -> bool
}
```

---

### P1.4 · `PgEventStore`

**Why:** `InMemoryEventStore` is for tests only.  Production needs Postgres.

**Schema (add to migrations):**

```sql
CREATE TABLE event_streams (
    stream_id      UUID    PRIMARY KEY,
    version        BIGINT  NOT NULL DEFAULT 0
);

CREATE TABLE events (
    global_offset  BIGSERIAL   PRIMARY KEY,
    stream_id      UUID        NOT NULL REFERENCES event_streams(stream_id),
    sequence       BIGINT      NOT NULL,
    payload        JSONB       NOT NULL,
    recorded_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (stream_id, sequence)
);
CREATE INDEX events_stream_id_idx ON events(stream_id, sequence);
```

**Implementation:** `PgEventStore` in `runtime/server/src/event_store.rs` implements
`EventStore` using `sqlx`.  The `append` method uses a transaction + `FOR UPDATE`
on `event_streams` for OCC.

---

### P1.5 · Character REST API

**Why:** Students and the frontend need to read and interact with characters.

**Endpoints (`runtime/server/src/routes/characters.rs`):**

```
GET    /characters/:id              → CharacterView
POST   /characters                  → spawn NPC (201 + CharacterView)
POST   /characters/:id/commands     → submit CharacterCommand (202)
GET    /characters/:id/events       → stream events (SSE, from checkpoint)
GET    /characters/:id/goals        → GoalStack summary
GET    /characters/:id/memory       → salient episodes
GET    /characters/:id/relationships → RelationshipGraph summary
```

---

## P2 — Future

### P2.1 · World simulation loop

Wire characters into a per-realm tick loop:

```rust
// runtime/server: background task
loop {
    let tick = WorldTick(counter.fetch_add(1, Ordering::SeqCst));
    let ctx  = TickContext { tick, realm, rng_seed: derive_seed(realm, tick), delta_ticks: 1 };
    for character in world.all_npcs() {
        let events = TickEngine::tick(&character, &ctx);
        store.append(StreamId::from_uuid(character.id.inner()),
                     ExpectedVersion::Exact(character.version),
                     serialize_events(events)).await?;
    }
    projection_bus.broadcast(events);
    tokio::time::sleep(tick_interval).await;
}
```

---

### P2.2 · Multi-realm support

Partition all aggregates by `RealmId`.  Students join a realm; their character
lives in that realm's tick loop.  Realms are isolated — events in one realm
never affect another.

---

### P2.3 · Snapshot + full replay

- Implement `DeterministicHash for Character` (serialize → SHA-256)
- Persist `WorldSnapshot` to Postgres when `version % SNAPSHOT_INTERVAL == 0`
- Rebuild aggregate from `(last_snapshot, events_since_snapshot)` instead of from epoch
- Cross-node verification: compare `state_hash` values across replicas

---

### P2.4 · Plugin system (cdylib)

Extract the plugin loading boilerplate into a `runtime/plugin-host` crate:

- Plugin manifest (`[plugin]` section in TOML: `id`, `kind`, `api_version`)
- Dynamic loading via `libloading`
- Capability declarations (what events a plugin subscribes to / emits)
- Hot-reload in dev mode

First candidate: `forgefabrik.llm-free` already has the right structure.

---

### P2.5 · LLM streaming (SSE)

Replace `FreeClient::chat()` (blocking, returns full string) with a streaming
version using Server-Sent Events.  Relevant for the hint endpoint where students
want to see the response appear word by word.

```rust
pub async fn chat_stream(&self, messages, max_tokens) -> impl Stream<Item = String>
```

All six free providers support `"stream": true` in the OpenAI-compatible request.

---

### P2.6 · Social engine

- `ConversationSystem` — matches NPCs for conversations based on schedule + proximity
- `FactionSystem` — tracks faction membership changes and their world effects
- `ReputationProjection` — read model: how does the world perceive this character?

---

### P2.7 · LLM-augmented goal selection (`domain/agents`)

The `AgentStrategy` trait already exists in `foundation/types`.  Wire it up:

1. `domain/agents::Orchestrator` produces a prompt from character state
2. `plugins/plugin-llm-free::FreeClient` sends it to the best available free model
3. Response is parsed into a `CharacterCommand::AssignGoal { ... }`
4. That command goes through `AggregateRoot::handle` — same validation, same reducer

The LLM only *suggests* goals.  The domain rules still guard correctness.

---

## Invariants to preserve forever

| Invariant | Where enforced |
|---|---|
| `domain/` never calls `Utc::now()` | Layer boundary |
| `domain/` never calls `thread_rng()` | Use `DeterministicRng` only |
| `domain/` never reads from DB or network | Layer boundary |
| Every state change = one `CharacterEvent` variant | `CharacterReducer` |
| `replay(initial, events) == sequential_apply(initial, events)` | `aggregate_replay_matches_sequential_application` test |
| `GoalId` in planner is deterministic (UUIDv5) | `planner_same_input_same_output` test |
| Stats never exceed declared max | `Stats::clamp()` called in reducer |
