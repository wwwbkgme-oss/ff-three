# `domain/characters`

The character simulation engine — the richest aggregate in the Academy.

**Layer rule:** Deterministic, replay-safe, no I/O.

## Architecture

```
CharacterCommand
    ↓  AggregateRoot::handle()   validate + emit events
Vec<CharacterEvent>
    ↓  CharacterReducer::apply() fold into state
Character (new state, version += 1)
```

## Modules

| Module | Responsibility |
|---|---|
| `character.rs` | `Character` struct — pure data, `version: u64` for OCC |
| `stats.rs` | `Stats` (health, energy, hunger, fatigue, social_need), `Mood` |
| `goals.rs` | `GoalType`, `GoalStack`, `Condition` — GOAP preconditions |
| `schedule.rs` | `Schedule`, `TimeSlot` — 2 400 ticks/day |
| `memory.rs` | `Episode` — weighted, deterministic decay |
| `relationships.rs` | `RelationshipGraph` — BTreeMap for stable iteration |
| `planner.rs` | `Planner::suggest()` — pure GOAP, UUIDv5 goal IDs |
| `tick.rs` | `TickEngine::tick()` — stat decay → goal inject → goal activate |
| `reducer.rs` | `AggregateRoot for Character` — full `handle` + `apply` |
| `commands.rs` | `CharacterCommand` enum |

## Tests

18 determinism tests in `tests/determinism.rs`:

```bash
cargo test -p characters
```

| Test | What it verifies |
|---|---|
| `planner_same_input_same_output` | GoalIds are UUIDv5 (deterministic) |
| `tick_engine_same_input_same_events` | Same `(character, ctx)` → same events |
| `reducer_version_increments` | version += 1 per applied event |
| `aggregate_replay_matches_sequential_application` | Replay-safety |

## Examples

```bash
cargo run --example basic_npc       -p characters
cargo run --example daily_schedule  -p characters
```
