# ForgeFabrik Academy — ff-three

An event-sourced, deterministic simulation engine for an educational RPG.

**Föderiertes ForgeFabrik-System:**  
[`forge-core`](https://github.com/wwwbkgme-oss/forge-core) ← Kanonischer Kernel |
[`FORGE_CORE_SYNC.md`](FORGE_CORE_SYNC.md) ← Compliance-Status
Characters evolve, pursue goals, form memories, and socialise — all driven
by a replay-safe event log.  An Axum HTTP server exposes the academy to
students; free-tier LLM drivers power AI agents without cloud spend.

> Full architecture: **[ARCHITECTURE.md](ARCHITECTURE.md)**
> Layer boundary spec: **[docs/PLUGIN_VS_DRIVER.md](docs/PLUGIN_VS_DRIVER.md)**
> Roadmap: **[NEXT.md](NEXT.md)**

---

## Repository layout

```
ff-three/
├── foundation/          Pure primitives — no I/O, no randomness
│   ├── types/           IDs, WorldTick, TickContext, DeterministicRng,
│   │                    AggregateRoot, Reducer, CommandContext, EventStore trait
│   └── events/          CharacterEvent, AcademyEvent, EventEnvelope,
│                        InMemoryEventStore, AcademyCommand
│
├── domain/              Business logic — deterministic, replay-safe
│   ├── characters/      Character aggregate: GOAP planner, TickEngine,
│   │                    CharacterReducer, CharacterCommand, 18 tests
│   ├── quests/          Quest lifecycle, rules, XP formulae
│   ├── world/           Biome state engine, knowledge graph
│   └── agents/          AgentStrategy implementations (pure, no I/O)
│
├── runtime/             I/O and infrastructure
│   ├── drivers/         I/O adapters  ← not plugins
│   │   └── llm/         FreeClient: Groq → SambaNova → LLM7 →
│   │                    OpenRouter → NVIDIA NIM → Ollama
│   ├── projections/     CharacterView read model
│   ├── server/          Axum HTTP, Postgres (PgEventStore), Redis,
│   │                    migration runner, REST routes
│   └── sandbox/         Code execution sandbox
│
├── plugins/             Domain-behaviour extensions (reserved, empty)
│                        See docs/PLUGIN_VS_DRIVER.md for what goes here
│
├── migrations/          5 PostgreSQL migrations
├── infra/               Pulumi TypeScript (AWS)
└── docs/                Architecture decision records
```

### The critical boundary

```
plugins/         = domain-behaviour extensions  (pure, no I/O)
runtime/drivers/ = infrastructure I/O adapters  (HTTP, storage, …)
```

These are **not interchangeable**.  See [`docs/PLUGIN_VS_DRIVER.md`](docs/PLUGIN_VS_DRIVER.md).

---

## Getting started

### Prerequisites

- Rust stable ≥ 1.75
- Docker + Docker Compose

### Start the local stack

```bash
docker-compose up -d
cp .env.example .env
# edit .env — required: DATABASE_URL, REDIS_URL, JWT_SECRET
cargo run -p server
```

### Enable LLM features (free, no OpenAI account needed)

Set **any** of these env vars — `FreeClient` auto-detects and chains them:

```bash
# Option A — Groq (fastest, 14 400 req/day)
export GROQ_API_KEY=gsk_...         # console.groq.com

# Option B — SambaNova (no credit card)
export SAMBANOVA_API_KEY=...        # cloud.sambanova.ai

# Option C — LLM7 free gateway
export LLM7_API_KEY=...             # token.llm7.io

# Option D — OpenRouter :free models
export OPENROUTER_API_KEY=sk-or-... # openrouter.ai/keys

# Option E — NVIDIA NIM (1 000 req/month)
export NVIDIA_API_KEY=nvapi-...     # build.nvidia.com

# Option F — Ollama (fully local, no account)
ollama pull llama3.2 && ollama serve
```

Multiple providers can be active simultaneously — the driver tries them in
the order above, falling back silently on 429s.

### Run tests

```bash
cargo test -p characters       # 18 determinism tests
cargo test                     # full workspace
```

### Production-Ready Features

| Feature | Status |
|---|---|
| Character REST handlers (spawn/get/tick/goals/memory) | ✓ via PgEventStore + CharacterReducer |
| Tick-Worker (1 Hz Simulation) | ✓ `sim/tick_worker.rs` — startet mit Server |
| Projection-Worker (Read-Model catch-up) | ✓ `sim/projection_worker.rs` |
| 6 Migrations (inkl. 006_projections) | ✓ |
| CI/CD (GitHub Actions) | ✓ `.github/workflows/ci.yml` |
| Free-LLM-Drivers (Groq, SambaNova, LLM7, OpenRouter, NVIDIA, Ollama) | ✓ |
| Event-First + OCC-Guard | ✓ `ExpectedVersion::Exact(version)` |
| SYNC_CONTRACT v0.1 Compliance | ✓ `FORGE_CORE_SYNC.md` |

### Run examples

```bash
cargo run --example basic_npc       -p characters
cargo run --example daily_schedule  -p characters
```

---

## Environment variables

### Required

| Variable       | Example                                 | Description          |
|----------------|-----------------------------------------|----------------------|
| `DATABASE_URL` | `postgres://user:pass@localhost/forge`  | PostgreSQL DSN       |
| `REDIS_URL`    | `redis://localhost:6379`                | Redis DSN            |
| `JWT_SECRET`   | 64-byte hex (`openssl rand -hex 64`)    | Token signing secret |

### LLM providers (one or more recommended)

| Variable             | Provider     | Free tier                      |
|----------------------|--------------|--------------------------------|
| `GROQ_API_KEY`       | Groq Cloud   | 14 400 req/day, <100 ms TTFT   |
| `SAMBANOVA_API_KEY`  | SambaNova    | 20–480 RPM, no credit card     |
| `LLM7_API_KEY`       | LLM7.io      | 100 req/hr, free token         |
| `OPENROUTER_API_KEY` | OpenRouter   | `:free` models, $0/token       |
| `NVIDIA_API_KEY`     | NVIDIA NIM   | 1 000 req/month credits        |
| `OLLAMA_HOST`        | Ollama local | always free, `ollama serve`    |

### Optional

| Variable                | Default | Description                  |
|-------------------------|---------|------------------------------|
| `APP_HOST`              | 0.0.0.0 | Listen address               |
| `APP_PORT`              | 8080    | Listen port                  |
| `SANDBOX_TIMEOUT_SECS`  | 30      | Max wall time per submission |
| `SANDBOX_MAX_MEMORY_MB` | 128     | Max RSS per submission       |
| `OLLAMA_HOST`           | http://localhost:11434 | Ollama base URL |

---

## Database

Five migrations in `migrations/`:

| File | Content |
|---|---|
| `001_enum_types.sql` | Postgres enums: `biome_domain`, `quest_type`, … |
| `002_tables.sql` | students, biomes, quests, sandbox\_runs, groups, achievements |
| `003_indexes.sql` | Performance indexes |
| `004_seed_biomes.sql` | Six starting biomes (Python, Algorithms, Systems, …) |
| `005_event_store.sql` | `event_streams` + `events` (append-only log) |

Apply: `sqlx migrate run` (requires `DATABASE_URL`).

---

## Key concepts

### WorldTick

`WorldTick(u64)` — monotonic simulation counter, replaces all wall-clock usage
inside `domain/`.  One tick = one logical step; duration is runtime-defined
(default: 1 Hz → 2 400 ticks/day).

### TickContext

```rust
pub struct TickContext {
    pub tick:        WorldTick,  // absolute counter
    pub realm:       RealmId,    // world shard
    pub rng_seed:    u64,        // deterministic RNG seed for this tick
    pub delta_ticks: u64,        // 1 normally; larger during catch-up
}
```

Passed to `TickEngine::tick()` and `Planner::suggest()`.  The domain never
calls `Utc::now()` or `thread_rng()` — all non-determinism enters through here.

### Character.version

Every `Character` carries `version: u64` (= events applied since birth).
Used as the optimistic concurrency guard for `PgEventStore::append`.

### DeterministicRng

Xorshift64* PRNG seeded from `(global_seed, tick, entity_id)`.  Use this
anywhere in `domain/` where randomness is needed.  **Never** call
`rand::thread_rng()` inside `domain/`.

---

## Architecture references

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — full system architecture
- [`docs/PLUGIN_VS_DRIVER.md`](docs/PLUGIN_VS_DRIVER.md) — frozen boundary spec
- [`NEXT.md`](NEXT.md) — roadmap and P2 sprint plan

---

## License

MIT
