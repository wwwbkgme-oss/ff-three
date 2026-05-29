# Changelog — ff-three (ForgeFabrik Academy)

---

## [Unreleased] — `neo/workspace-restructure-a8f3k`

### Production-ready features added

| Feature | Commit(s) |
|---|---|
| **EventStore trait + InMemoryEventStore** | `a6b5ec4` |
| **PgEventStore** — Postgres append-only log, OCC guard | `1a53d6a` |
| **Migration 005** — `event_streams` + `events` tables | `1c160a3` |
| **Migration 006** — `character_views` projection table | `792a427` |
| **Character.version** — optimistic concurrency field | `94ec9b0` |
| **CharacterReducer** increments version on every event | `c453afa` |
| **TickContext + DeterministicRng** — deterministic domain entry-points | `dde74ce` |
| **Planner + TickEngine** use TickContext (no bare WorldTick) | `f87dd42` |
| **Character REST API** — 5 routes fully wired through PgEventStore | `9641d15` |
| **Tick worker** — background task: TickEngine + PgEventStore per NPC tick | `cc6df8d` |
| **Projection worker** — background task: CharacterView catch-up from event log | `792a427` |
| **runtime/projections** crate — CharacterView incremental read model | `da0fe54` |
| **runtime/drivers/llm** — FreeClient: 6 free-tier LLM providers, auto-failover | `8102a60` |
| **LLM moved from plugins/ to runtime/drivers/** (arch refactor) | `8f5602d` |
| **ARCHITECTURE.md** — full system architecture | `ba3b2dc` |
| **docs/PLUGIN_VS_DRIVER.md** — frozen boundary specification | `47da5b5` |
| **AGENTS.md** — federation instructions | `6e7e946` |
| **FORGE_CORE_SYNC.md** — compliance with forge-core SYNC_CONTRACT v0.1 | `492b96f` |
| **CI** — GitHub Actions: check, clippy, 18 tests, infra TS | `7b8405f` |

### Architecture

- Layer model: `foundation → domain → runtime → plugins` (enforced)
- 18 determinism tests in `domain/characters/tests/determinism.rs`
- `CharacterEvent::Created` persisted on spawn; full event replay on every read
- `TickContext::new(tick, realm_uuid, delta)` — production constructor
- `TickContext::test(tick)` — fixed realm UUID, fully deterministic
- Plugin boundary: `plugins/` reserved for pure domain behaviour (empty)
- `runtime/drivers/` is the I/O adapter layer (LLM, future: storage, notifications)

### forge-core SYNC_CONTRACT compliance

| § | Rule | Status |
|---|---|---|
| §2 | 4-layer model | ✅ |
| §3 | `WorldTick`, `TickContext`, `DeterministicRng`, `WorldSnapshot` | ✅ |
| §4 | No `Utc::now()` / `thread_rng()` in domain | ✅ |
| §5 | Event-first, single mutation path, EventStore | ✅ |
| §6 | Plugin ABI | ⬜ planned P2.8 |
| §8 | 18 determinism tests | ✅ |
