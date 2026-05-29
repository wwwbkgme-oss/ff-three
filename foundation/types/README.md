# `foundation/types`

Single source of truth for all primitive domain types.

**Layer rule:** No I/O · no business logic · no randomness · only types, IDs, traits, and errors.

## Key exports

| Type | Description |
|---|---|
| `WorldTick(u64)` | Monotonic simulation counter — replaces all `Utc::now()` in domain |
| `TickContext` | Tick + realm + rng_seed + delta — injected into every domain entry-point |
| `DeterministicRng` | Xorshift64* PRNG, seeded from `(global_seed, tick, entity_id)` |
| `WorldSnapshot` | Deterministic checkpoint with `state_hash: [u8; 32]` |
| `CharacterId`, `GoalId`, `LocationId`, … | Type-safe UUID newtypes via `define_id!` macro |
| `AggregateRoot` | Trait: `handle(command) → Vec<Event>` + `apply(state, event) → state` |
| `Reducer<S, E>` | Trait: `apply(state, event) → state` |
| `CommandContext` | Tick + actor + realm — injected into every command handler |
| `ForgeError`, `ForgeResult<T>` | Canonical error types |

## Determinism contract

```rust
// Forbidden in domain/ and foundation/
chrono::Utc::now()      // ← use WorldTick
rand::thread_rng()      // ← use DeterministicRng

// TickContext is the only permitted non-determinism entry-point
pub fn domain_fn(ctx: &TickContext) { ... }
```
