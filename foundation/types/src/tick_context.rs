//! Deterministic tick context — replaces bare `WorldTick` at domain entry-points.
//!
//! Bundles everything a tick operation may legitimately observe:
//! simulation time, realm identity, and a deterministic RNG seed.
//! Nothing non-deterministic enters the domain except through this struct.

use serde::{Deserialize, Serialize};

use crate::{ids::RealmId, time::WorldTick};

/// Immutable context for one simulation tick.
///
/// Passed by reference into `TickEngine::tick`, `Planner::suggest`, and any
/// domain function that needs the current time or deterministic randomness.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TickContext {
    /// Absolute simulation tick.
    pub tick:        WorldTick,
    /// World shard this tick belongs to.
    pub realm:       RealmId,
    /// Deterministic seed for this tick's `DeterministicRng`.
    ///
    /// Runtime derives this as `realm_seed ^ tick.0` so every tick in every
    /// realm has a unique, reproducible seed.
    pub rng_seed:    u64,
    /// Ticks elapsed since the previous `TickContext`.
    ///
    /// Usually 1.  Can be larger during catch-up or time-compressed
    /// simulation (fast-forward, load-from-snapshot).
    pub delta_ticks: u64,
}

impl TickContext {
    /// Full constructor: supply tick, realm UUID, and delta.
    ///
    /// `rng_seed` is derived deterministically from `tick ^ realm`.
    pub fn new(tick: u64, realm_uuid: uuid::Uuid, delta_ticks: u64) -> Self {
        use crate::ids::RealmId;
        let realm = RealmId::from(realm_uuid);
        let rng_seed = tick
            .wrapping_mul(0x9e3779b97f4a7c15)
            ^ (realm_uuid.as_u128() as u64);
        Self { tick: WorldTick(tick), realm, rng_seed, delta_ticks }
    }

    /// Minimal constructor for unit tests and single-realm deployments.
    ///
    /// Uses a fixed realm UUID so the seed is deterministic.
    pub fn test(tick: u64) -> Self {
        // Fixed realm UUID for tests — never call new_v4() here.
        const TEST_REALM: uuid::Uuid = uuid::Uuid::from_u128(0x_dead_beef_0000_0000_0000_0000_0000_0001);
        Self::new(tick, TEST_REALM, 1)
    }

    /// Advance to the next tick, refreshing the RNG seed.
    pub fn advance(self) -> Self {
        let next = self.tick.advance(1);
        Self {
            tick:        next,
            rng_seed:    next.0.wrapping_mul(0x9e3779b97f4a7c15) ^ self.rng_seed,
            delta_ticks: 1,
            ..self
        }
    }
}
