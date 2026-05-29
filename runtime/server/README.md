# `runtime/server`

Axum HTTP server — wires domain + drivers + event store together.

## Startup sequence

```
AppConfig::from_env()
    ↓
AppState::new()  — connect Postgres, Redis, build PgEventStore + FreeClient
    ↓
sqlx migrate run
    ↓
spawn background workers: TickWorker + ProjectionWorker
    ↓
axum::serve(router, app)
```

## Routes

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Liveness check |
| `GET` | `/readiness` | Readiness check |
| `POST` | `/characters` | Spawn NPC — persists `CharacterEvent::Created` |
| `GET` | `/characters/:id` | Replay stream → current state |
| `POST` | `/characters/:id/tick` | TickEngine + persist events (OCC guard) |
| `GET` | `/characters/:id/goals` | Goal stack summary |
| `GET` | `/characters/:id/memory` | Salient episodes |
| `POST` | `/students/enroll` | Enroll student |
| `GET` | `/biomes` | List biomes |
| `POST` | `/quests/generate` | Generate quest (uses LLM if available) |
| `POST` | `/sandbox/run` | Execute code submission |

## Background workers

| Worker | File | What it does |
|---|---|---|
| `TickWorker` | `sim/tick_worker.rs` | Runs `TickEngine::tick` for all NPCs every tick interval |
| `ProjectionWorker` | `sim/projection_worker.rs` | Rebuilds `CharacterView` from event log |

## Key files

```
src/
├── main.rs           — entry point
├── config.rs         — AppConfig from env vars
├── state.rs          — AppState (db, redis, store, orchestrator, llm)
├── event_store.rs    — PgEventStore (transaction + OCC)
├── error.rs          — ServerError → HTTP status codes
├── routes/
│   ├── characters.rs — production character handlers
│   └── ...
└── sim/
    ├── tick_worker.rs        — background tick loop
    └── projection_worker.rs  — background projection catch-up
```

## Environment variables

See [`.env.example`](../../.env.example) for the full list.
Required: `DATABASE_URL`, `REDIS_URL`, `JWT_SECRET`.
LLM: set any of `GROQ_API_KEY`, `SAMBANOVA_API_KEY`, etc.
