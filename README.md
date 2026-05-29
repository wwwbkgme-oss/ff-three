# ForgeFabrik Academy — ff-three

An event-sourced, deterministic simulation engine for an educational RPG.
Characters evolve, pursue goals, form memories, and socialize — all driven
by a replay-safe event log.  An Axum HTTP server exposes the academy to
students; a free-tier LLM plugin powers AI agents without cloud spend.

---

## Repository layout

```
ff-three/
├── foundation/          Pure primitives — no I/O, no side effects
│   ├── types/           IDs, WorldTick, traits (Reducer, AggregateRoot), snapshots
│   └── events/          AcademyEvent, CharacterEvent, EventEnvelope, AcademyCommand
│
├── domain/              Business logic — depends only on foundation
│   ├── characters/      Character aggregate: GOAP planner, TickEngine, reducer, 17 tests
│   ├── quests/          Quest generation and progression
│   ├── world/           World state and knowledge graph
│   └── agents/          AI agent strategies (pure, no I/O)
│
├── runtime/             I/O and infrastructure — depends on foundation + domain
│   ├── server/          Axum HTTP server, Postgres repos, Redis cache, LLM client
│   └── sandbox/         Code execution sandbox
│
├── plugins/             Runtime-loadable extensions
│   └── plugin-llm-free/ Free-tier LLM provider chain (6 providers, auto-failover)
│
├── infra/               Pulumi TypeScript — AWS infrastructure
├── migrations/          4 PostgreSQL migrations (enum types, tables, indexes, seed)
└── docker-compose.yml   Local dev stack (Postgres + Redis)
```

### Naming convention for plugins

| Layer | Convention | Example |
|---|---|---|
| Folder | `plugins/plugin-{name}/` | `plugins/plugin-llm-free/` |
| Crate name | `{name}` (no `plugin-` prefix) | `llm-free` |
| Plugin ID | `forgefabrik.{name}` | `forgefabrik.llm-free` |

---

## Architecture principles

### Events are truth. State is projection.

```
Command
  └─► CharacterCommandHandler::handle()   ← validates, emits events
          └─► Vec<CharacterEvent>
                  └─► CharacterReducer::apply()   ← pure fold
                          └─► Character (new state)
```

Every mutation is a fact in the event log.  State can be rebuilt from
scratch by replaying events.  No direct mutations — ever.

### Determinism first

| Rule | Where enforced |
|---|---|
| No `Utc::now()` in domain | `foundation/types` layer boundary |
| No `rand::thread_rng()` in domain | Use `DeterministicRng` (planned, see NEXT.md) |
| No random IDs in tick logic | `Planner` derives GoalIds via UUIDv5 from `(char_id, tick, discriminant)` |
| Same `(character, tick)` → same events | 17 determinism tests in `domain/characters/tests/` |

### Layer boundaries

```
foundation/types   ← no deps outside std + serde + sqlx (serialisation only)
foundation/events  ← depends on types only
domain/*           ← depends on foundation only, NO I/O
runtime/*          ← may depend on everything, I/O lives here
plugins/*          ← may depend on foundation + domain, I/O allowed
```

---

## Crates

### `foundation/types`

Primitive domain types, IDs, traits, simulation time.

- **`WorldTick(u64)`** — monotonic simulation counter; replaces all wall-clock usage inside `domain/`
- **`define_id!(Name)`** macro — generates 12 ID newtypes (`CharacterId`, `GoalId`, `LocationId`, …)
- **`Reducer<S, E>`** — apply one event to state deterministically
- **`AggregateRoot`** — `handle` (validate command → emit events) + `apply` (project events → state) + `replay`
- **`CommandContext`** — tick + actor + realm + correlation injected into every command handler
- **`WorldSnapshot`** + **`DeterministicHash`** — checkpoint and verify world state

### `foundation/events`

All emitted facts.

- **`CharacterEvent`** — 18 variants: lifecycle, movement, goals, memory, social, factions, mood, stats
- **`AcademyEvent`** — student enroll, quest start/complete, XP, biome, groups, achievements
- **`EventEnvelope<E>`** — wraps every event with `event_id`, `causation_id`, `correlation_id`, `tick`, `actor`, `realm`
- **`AcademyCommand`** — intents from students and the system

### `domain/characters`

The character simulation engine.

| Module | Responsibility |
|---|---|
| `character.rs` | `Character` aggregate struct — pure data, no side effects |
| `stats.rs` | `Stats` (health, energy, hunger, fatigue, social\_need), `Mood` |
| `goals.rs` | `GoalType`, `GoalStack`, `Condition` — GOAP preconditions |
| `schedule.rs` | `Schedule`, `TimeSlot`, `ScheduledActivity` — daily routine (2 400 ticks/day) |
| `memory.rs` | `Episode`, `Memory` — weighted, decaying memory with deterministic decay |
| `relationships.rs` | `RelationshipGraph` — directed trust/affinity edges |
| `planner.rs` | `Planner::suggest()` — pure GOAP: selects goals from state, deterministic UUIDv5 IDs |
| `tick.rs` | `TickEngine::tick()` — stat decay → goal injection → goal activation per tick |
| `reducer.rs` | `AggregateRoot for Character` — full `handle` + `apply` + `CharacterReducer` wrapper |
| `commands.rs` | `CharacterCommand` enum — move, goals, social, memory, factions |

**Tests:** `domain/characters/tests/determinism.rs` — 17 tests covering planner determinism,
reducer invariants, stat clamping, goal lifecycle, memory decay, command round-trips, replay.

**Examples:**
- `cargo run --example basic_npc -p characters`
- `cargo run --example daily_schedule -p characters`

### `plugins/plugin-llm-free`

Drop-in replacement for `runtime/server/src/llm.rs` backed by free-tier providers only.

**Provider chain (priority order):**

| # | Provider | Free tier | Env var |
|---|---|---|---|
| 1 | **Groq** | 14 400 req/day · <100 ms TTFT | `GROQ_API_KEY` |
| 2 | **SambaNova** | 20–480 RPM · no credit card | `SAMBANOVA_API_KEY` |
| 3 | **LLM7** | 100 req/hr · free token | `LLM7_API_KEY` |
| 4 | **OpenRouter** | `:free` models · $0/token | `OPENROUTER_API_KEY` |
| 5 | **NVIDIA NIM** | 1 000 req/month credits | `NVIDIA_API_KEY` |
| 6 | **Ollama** | Local inference · always free | `ollama serve` |

`FreeClient::from_env()` auto-detects configured providers and builds the chain.
`chat()` silently falls through to the next provider on 429 / errors.

Same API as `LlmClient`: `chat()`, `run_quest_generation()`, `run_evaluation()`, `run_hint()`.

---

## Getting started

### Prerequisites

- Rust stable (≥ 1.75)
- Docker + Docker Compose (for Postgres + Redis)

### Run the local stack

```bash
docker-compose up -d
cp .env.example .env
# Edit .env — set DATABASE_URL, REDIS_URL, and at least one LLM key
cargo run -p server
```

### Use a free LLM provider (no OpenAI key needed)

```bash
# Option A — Groq (fastest, 14 400 req/day free)
export GROQ_API_KEY=gsk_...     # console.groq.com

# Option B — SambaNova (no credit card)
export SAMBANOVA_API_KEY=...    # cloud.sambanova.ai

# Option C — Ollama (fully local)
ollama pull llama3.2
ollama serve
```

Then replace `LlmClient::new(...)` in `runtime/server` with `FreeClient::from_env()`.

### Run character tests

```bash
cargo test -p characters
```

### Run the examples

```bash
cargo run --example basic_npc       -p characters
cargo run --example daily_schedule  -p characters
```

---

## Database

Four migrations in `migrations/`:

| File | Content |
|---|---|
| `001_enum_types.sql` | Postgres enums: `biome_domain`, `biome_state`, `quest_type`, … |
| `002_tables.sql` | students, biomes, quests, sandbox\_runs, groups, achievements, certifications |
| `003_indexes.sql` | Performance indexes |
| `004_seed_biomes.sql` | Seed biomes (Python, Algorithms, Systems, Web, Data, Security) |

Apply with `sqlx migrate run` (DATABASE\_URL must be set).

---

## Infrastructure

`infra/` is a Pulumi TypeScript stack targeting AWS.  See `infra/README.md`.

---

## License

MIT
