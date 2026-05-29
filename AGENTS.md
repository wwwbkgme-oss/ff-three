# AGENTS.md — ff-three (ForgeFabrik Academy)

**Treat this repository as part of a federated ForgeFabrik system.**

Maintain compatibility with the canonical event + domain model defined in
[forge-core](https://github.com/wwwbkgme-oss/forge-core).
Do not introduce incompatible abstractions.

---

## This repo's role

`ff-three` is the **Academy / Event-sourced educational simulation** node of
the ForgeFabrik federation.

| Responsibility | Location |
|---|---|
| Character aggregate (GOAP planner, 18 tests) | `domain/characters` |
| Quest lifecycle and XP formulae | `domain/quests` |
| Biome state engine, knowledge graph | `domain/world` |
| Agent strategy implementations (pure) | `domain/agents` |
| Free-tier LLM driver (FreeClient) | `runtime/drivers/llm` |
| Event projections (read models) | `runtime/projections` |
| Axum HTTP, Postgres (PgEventStore), Redis | `runtime/server` |
| Code execution sandbox | `runtime/sandbox` |
| PostgreSQL event store migrations | `migrations/` |
| AWS IaC (Pulumi TS) | `infra/` |

---

## Canonical types — ff-three is the reference implementation

`ff-three` has the most complete implementation of the canonical primitives.
Other repos should follow its patterns:

| Canonical name | Local type | File |
|---|---|---|
| `WorldTick` | `WorldTick` | `foundation/types/src/time.rs` |
| `TickContext` | `TickContext` | `foundation/types/src/tick_context.rs` |
| `DeterministicRng` | `DeterministicRng` | `foundation/types/src/rng.rs` |
| `WorldSnapshot` | `WorldSnapshot` | `foundation/types/src/snapshot.rs` |
| `EventEnvelope<E>` | `EventEnvelope` | `foundation/events/src/envelope.rs` |
| `EventStore<E>` | `EventStore` trait | `foundation/events/src/store.rs` |
| `ForgeError` | `ForgeError` | `foundation/types/src/errors.rs` |

---

## Layer rules

```
foundation → domain → runtime → plugins
```

| Layer | Allowed | Forbidden |
|---|---|---|
| `foundation/` | types, events, errors, traits | I/O, randomness, business logic |
| `domain/` | deterministic logic, GOAP, reducers | HTTP, DB, `Utc::now()`, `thread_rng()` |
| `runtime/` | I/O, HTTP, Postgres, Redis, drivers | domain business logic |
| `plugins/` | domain-behaviour extensions (reserved, empty) | I/O of any kind |

---

## Plugin vs Driver boundary

```
plugins/         = domain-behaviour extensions  (pure, no I/O)
runtime/drivers/ = infrastructure I/O adapters  (HTTP, storage, …)
```

Reference: [`docs/PLUGIN_VS_DRIVER.md`](docs/PLUGIN_VS_DRIVER.md) (equivalent
to forge-core's `docs/DRIVER_PLUGIN_BOUNDARY.md`).

---

## Event-First mandate

```
Events are truth. State is projection.
Command → Event → Reducer → State Projection
```

`ff-three` has a full Postgres-backed `EventStore` (append-only log) and
event projections. This is the target architecture for the other repos.

---

## Determinism rules

`ff-three` is the reference for determinism:

- `WorldTick` replaces all wall-clock usage in `domain/`.
- `DeterministicRng` (Xorshift64*) seeded from `(global_seed, tick, entity_id)`.
- `TickContext` is the only source of non-determinism injected into domain code.

Forbidden in `domain/` and `foundation/`:

- `chrono::Utc::now()` used to affect state
- `rand::thread_rng()` — use `DeterministicRng` instead
- Global mutable state

---

## Testing requirement

`ff-three` sets the standard: **18 deterministic tests** in `domain/characters`.

Every repo must have:
- Deterministic replay test: `replay(events) == snapshot.state_hash`
- Event equality test
- Snapshot roundtrip test

---

## Federation links

- [`forge-core`](https://github.com/wwwbkgme-oss/forge-core) — canonical definitions
- [`docs/SYNC_CONTRACT.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/SYNC_CONTRACT.md) — federation-wide sync contract
- [`docs/PLUGIN_ABI.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/docs/PLUGIN_ABI.md) — canonical plugin ABI
