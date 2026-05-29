# ForgeFabrik Academy — Architecture (ff-three)

> **Events are truth. State is projection.**

> forge-core kernel: [forge-core/SYNC_CONTRACT.md](https://github.com/wwwbkgme-oss/forge-core/blob/main/SYNC_CONTRACT.md)  
> Compliance: [FORGE_CORE_SYNC.md](FORGE_CORE_SYNC.md)  
> ff-three contributes: `TickContext`, `DeterministicRng`, `AggregateRoot`, 18 determinism tests

This document describes the full system architecture.  For the Plugin vs Driver
boundary specifically, see [`docs/PLUGIN_VS_DRIVER.md`](docs/PLUGIN_VS_DRIVER.md).

---

## Layer overview

```
┌─────────────────────────────────────────────────────────────────┐
│  foundation/                                                    │
│    types    — IDs, WorldTick, TickContext, DeterministicRng,    │
│               Reducer, AggregateRoot, CommandContext trait       │
│    events   — CharacterEvent, AcademyEvent, EventEnvelope,       │
│               EventStore trait + InMemoryEventStore              │
│  No I/O.  No randomness.  No wall-clock time.                   │
└────────────────────────────┬────────────────────────────────────┘
                             │ depends on
┌────────────────────────────▼────────────────────────────────────┐
│  domain/                                                        │
│    characters — Character aggregate, GOAP planner, TickEngine,  │
│                 CharacterReducer, 18 determinism tests           │
│    quests     — Quest lifecycle, rules, XP calculation          │
│    world      — Biome state engine, knowledge graph             │
│    agents     — AgentStrategy implementations (pure, no I/O)    │
│  Deterministic.  Replay-safe.  Never imports drivers.           │
└────────────────────────────┬────────────────────────────────────┘
                             │ depends on
┌────────────────────────────▼────────────────────────────────────┐
│  runtime/                                                       │
│    drivers/  — I/O adapters                                     │
│      llm/    — FreeClient: Groq → SambaNova → LLM7 →           │
│                OpenRouter → NVIDIA NIM → Ollama                 │
│    projections/ — CharacterView read model                      │
│    server/   — Axum HTTP, Postgres (sqlx), Redis,               │
│                PgEventStore, character REST API                  │
│    sandbox/  — Code execution sandbox                           │
│  I/O allowed.  Wires domain + drivers together.                 │
└────────────────────────────┬────────────────────────────────────┘
                             │ (reserved, empty)
┌────────────────────────────▼────────────────────────────────────┐
│  plugins/  — Domain-behaviour extensions                        │
│    (none yet — see NEXT.md P2.4 for plugin host design)         │
│  Pure logic only.  No I/O.  No driver imports.                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Core design principles

### 1. Events are truth

Every state mutation is expressed as an immutable event variant and appended
to the event log.  State is a pure projection of events.  Nothing is mutated
in place.

```
Command
  └─► AggregateRoot::handle()   — validates, emits Vec<Event>
        └─► EventStore::append() — persisted to Postgres
              └─► Reducer::apply() — projects into current state
```

### 2. Determinism everywhere in domain

| Rule | Where enforced |
|---|---|
| No `Utc::now()` | `foundation/types` layer boundary |
| No `thread_rng()` | Use `DeterministicRng` from `types` |
| No DB/network calls | Layer boundary — domain never imports runtime |
| Same `(character, ctx)` → same events | 18 tests in `domain/characters/tests/` |
| GoalIds derived via UUIDv5 | `Planner::derive_id()` |

### 3. `TickContext` replaces bare time

All domain entry-points accept `&TickContext` instead of `WorldTick`:

```rust
pub struct TickContext {
    pub tick:        WorldTick,  // absolute simulation counter
    pub realm:       RealmId,    // world shard
    pub rng_seed:    u64,        // per-tick deterministic seed
    pub delta_ticks: u64,        // ticks since last context
}
```

The runtime constructs `TickContext`; the domain only reads it.

### 4. Optimistic concurrency via `Character.version`

Every `Character` carries a `version: u64` counter incremented by the reducer
after every applied event.  Before appending to the event store:

```rust
store.append(
    StreamId::from_uuid(char.id.inner()),
    ExpectedVersion::Exact(char.version),  // ← OCC guard
    serialised_events,
).await?;
```

A concurrent write returns `StoreError::ConcurrencyConflict` → caller retries.

---

## Data flow — one simulation tick

```
tick timer fires
    │
    ▼
TickContext { tick, realm, rng_seed, delta_ticks }
    │
    ├─► TickEngine::tick(&character, &ctx)
    │       1. passive stat deltas → StatsUpdated event
    │       2. Planner::suggest(&character, &ctx) → GoalAdded events
    │       3. goal activation check → GoalActivated event
    │
    ├─► PgEventStore::append(stream, ExpectedVersion::Exact(ver), events)
    │
    ├─► CharacterReducer::apply(character, events) → new Character state
    │       version += 1 per event
    │
    └─► CharacterView::apply(event, global_offset) → updated read model
            checkpoint = global_offset
```

---

## Event store

Two implementations:

| Impl | Usage |
|---|---|
| `InMemoryEventStore` | Unit tests, examples |
| `PgEventStore` | Production (migration 005) |

Schema:
```sql
event_streams (stream_id UUID PK, version BIGINT)
events (global_offset BIGSERIAL PK, stream_id, sequence, payload JSONB)
```

`global_offset` is the projection checkpoint.  `(stream_id, sequence)` is unique.

---

## LLM integration

LLM inference is a **driver** (I/O adapter), never a domain concern.

```
domain/agents (AgentStrategy — abstract, pure)
      ↑
      │  runtime wires concrete impl
      ↓
runtime/drivers/llm (FreeClient — HTTP, I/O)
      │
      ├─ Groq          GROQ_API_KEY          14 400 req/day free
      ├─ SambaNova     SAMBANOVA_API_KEY      20-480 RPM free
      ├─ LLM7          LLM7_API_KEY           100 req/hr free
      ├─ OpenRouter    OPENROUTER_API_KEY     :free models $0/token
      ├─ NVIDIA NIM    NVIDIA_API_KEY         1 000 req/month free
      └─ Ollama        (OLLAMA_HOST)          local, always free
```

`FreeClient::chat()` tries providers left to right; 429 → silent fallback.

Domain code accesses LLM only through `AppState.llm: Option<Arc<FreeClient>>`.
`ProviderKind` is never imported by `domain/` or `foundation/`.

See [`docs/PLUGIN_VS_DRIVER.md`](docs/PLUGIN_VS_DRIVER.md) for the full boundary spec.

---

## Character aggregate

The richest aggregate in the system.  Key modules:

| Module | Responsibility |
|---|---|
| `character.rs` | `Character` struct — pure data, `version: u64` |
| `stats.rs` | `Stats` (5 meters: health, energy, hunger, fatigue, social_need), `Mood` |
| `goals.rs` | `GoalType`, `GoalStack`, `Condition` — GOAP preconditions |
| `schedule.rs` | `Schedule`, `TimeSlot` — 2 400 ticks/day |
| `memory.rs` | `Episode` — weighted, decaying; `Memory` cap + `apply_decay()` |
| `relationships.rs` | `RelationshipGraph` — BTreeMap for deterministic iteration |
| `planner.rs` | `Planner::suggest()` — pure GOAP, UUIDv5 GoalIds |
| `tick.rs` | `TickEngine::tick()` — stat decay → goal inject → activate |
| `reducer.rs` | `AggregateRoot for Character` — `handle` + `apply` |
| `commands.rs` | `CharacterCommand` enum |

---

## REST API

```
GET  /health
GET  /readiness
GET  /students/:id
POST /students/enroll
GET  /biomes
POST /quests/generate
POST /quests/:id/submit
POST /characters              201  spawn NPC
GET  /characters/:id          200  CharacterView
POST /characters/:id/tick     200  advance one tick
GET  /characters/:id/goals    200  GoalStack summary
GET  /characters/:id/memory   200  salient episodes
```

---

## Infrastructure

- **Database:** PostgreSQL 15 (via sqlx 0.7, 5 migrations)
- **Cache:** Redis (session data, rate limiting)
- **Runtime:** Tokio 1, Axum 0.7
- **Observability:** tracing + tracing-subscriber
- **IaC:** Pulumi TypeScript (AWS, see `infra/`)
- **Local dev:** `docker-compose up` (Postgres + Redis)

---

## File tree

```
ff-three/
├── foundation/
│   ├── types/      IDs, WorldTick, TickContext, DeterministicRng, traits
│   └── events/     CharacterEvent, AcademyEvent, EventStore, EventEnvelope
├── domain/
│   ├── characters/ GOAP planner, TickEngine, CharacterReducer, 18 tests
│   ├── quests/     Quest lifecycle + rules
│   ├── world/      Biome state engine
│   └── agents/     AgentStrategy implementations
├── runtime/
│   ├── drivers/    I/O adapters
│   │   └── llm/    FreeClient (6 providers)
│   ├── projections/ CharacterView read model
│   ├── server/     Axum server, PgEventStore, REST routes
│   └── sandbox/    Code execution sandbox
├── plugins/        (empty — reserved for domain-behaviour extensions)
├── migrations/     5 SQL migration files
├── infra/          Pulumi TypeScript (AWS)
├── docs/
│   └── PLUGIN_VS_DRIVER.md  ← frozen boundary spec
├── ARCHITECTURE.md          ← this file
├── NEXT.md                  ← roadmap
└── README.md                ← getting started
```
