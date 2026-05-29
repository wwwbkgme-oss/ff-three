# forge-core Sync Contract — ff-three Compliance

Reference: [`forge-core/SYNC_CONTRACT.md`](https://github.com/wwwbkgme-oss/forge-core/blob/main/SYNC_CONTRACT.md) v0.1

> **Treat this repository as part of a federated ForgeFabrik system.**
> Maintain compatibility with the canonical event + domain model.
> Do not introduce incompatible abstractions.

---

## Compliance Status

| §   | Rule | Status | Notes |
|-----|------|--------|-------|
| §2  | Layer model: foundation ← domain ← runtime | ✅ | |
| §2  | Plugin = behavior only (no I/O) | ✅ | plugins/ reserved, empty |
| §2  | Driver = I/O adapter in runtime/drivers | ✅ | `runtime/drivers/llm/FreeClient` |
| §3  | `WorldTick` canonical name | ✅ | `foundation/types/src/time.rs` |
| §3  | `TickContext` canonical struct | ✅ | `foundation/types/src/tick_context.rs` |
| §3  | `DeterministicRng` | ✅ | `foundation/types/src/rng.rs` |
| §3  | `WorldEvent` canonical name | ⚠️ | Uses `CharacterEvent` + `AcademyEvent` — adapter needed |
| §3  | `ForgeFabrikPlugin` trait | ⬜ | Plugin host not yet implemented (NEXT.md P2.8) |
| §4  | No `Utc::now()` in domain/foundation | ✅ | Enforced via `TickContext` |
| §4  | No `thread_rng()` — use `DeterministicRng` | ✅ | `DeterministicRng::from_context(ctx)` |
| §4  | No global mutable state in foundation/domain | ✅ | |
| §5  | Event-First: Command → Event → Reducer → State | ✅ | `AggregateRoot::handle + apply` |
| §5  | Single mutation path | ✅ | `PgEventStore::append` + `CharacterReducer::apply` |
| §5  | `EventStore` trait | ✅ | `InMemoryEventStore` + `PgEventStore` |
| §5  | Optimistic concurrency (`ExpectedVersion`) | ✅ | `Character.version` OCC guard |
| §6  | Plugin ABI: `ff_plugin_init / tick / shutdown` | ⬜ | Geplant: NEXT.md P2.8 |
| §7  | `AgentKind::Free(FreeProvider)` | ✅ | `ProviderKind` in `runtime/drivers` |
| §8  | Deterministic replay test | ✅ | 18 Tests in `domain/characters/tests/` |
| §8  | Event equality test | ✅ | |
| §8  | Snapshot round-trip test | ✅ | |

**Legend:** ✅ compliant · ⚠️ partial/adapter needed · ⬜ planned

---

## Unique contributions of ff-three

These patterns from ff-three are canonical references for event sourcing:

- **`TickContext`** — richste Definition: tick + realm + rng_seed + delta_ticks
- **`DeterministicRng`** — xorshift64, `from_context(ctx)`
- **`AggregateRoot` trait** — `handle(Command) → Vec<Event>` + `apply(Event)`
- **GOAP planner** — goal-oriented action planning, UUIDv5 GoalIds
- **`PgEventStore`** — production-grade PostgreSQL event store
- **18 determinism tests** — reference test suite for §8 compliance
- **`CharacterReducer`** — reference Reducer implementation

---

## Canonical Event Mapping

ff-three uses domain-specific events. Canonical mapping for cross-repo integration:

| ff-three Event | forge-core canonical | Notes |
|---|---|---|
| `CharacterEvent::StatsUpdated` | `WorldEvent::AgentStateChanged` | via compatibility adapter |
| `CharacterEvent::GoalAdded` | `WorldEvent::AgentStateChanged` | |
| `CharacterEvent::GoalActivated` | `WorldEvent::AgentStateChanged` | |
| `AcademyEvent::StudentEnrolled` | `WorldEvent::AgentSpawned` | |
| `AcademyEvent::QuestCompleted` | `WorldEvent::EpochEnded` | semantic equivalent |

Compatibility adapters MUST live in `runtime/` — never in `domain/`.

---

## Pending Items

| Item | Priority | Reference |
|---|---|---|
| Plugin host (`runtime/plugin`) | P2.8 | NEXT.md |
| `ForgeFabrikPlugin` trait impl | P2.8 | NEXT.md |
| Plugin.toml manifest format | P2.8 | forge-core §6 |

The plugin system follows ff-one's reference implementation.
See `forge-core/SYNC_CONTRACT.md` §6 for the full ABI spec.
