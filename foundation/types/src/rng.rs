//! Deterministic PRNG for domain use.
//!
//! **Rule: `rand::thread_rng()` is FORBIDDEN in `domain/`.**
//!
//! Any randomness in domain logic (goal priority jitter, loot rolls, mood
//! noise) MUST come through `DeterministicRng`.  This guarantees:
//!
//! - Replay correctness: same seed → same sequence, always.
//! - Cross-node determinism: two nodes with the same seed produce the same world.
//! - Test reproducibility: failures can be reproduced from the seed alone.
//!
//! ## Algorithm
//!
//! xorshift64* — fast, passes BigCrush, period 2^64 − 1.
//!
//! ## Seeding
//!
//! ```rust
//! use types::{DeterministicRng, WorldTick};
//! let mut rng = DeterministicRng::new(ctx.rng_seed, ctx.tick, char_id.inner().as_u128());
//! let jitter: f64 = rng.next_f64();
//! ```

use crate::time::WorldTick;

/// Deterministic pseudo-random number generator seeded from simulation context.
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    /// Seed from `(global_seed, tick, entity_id)`.
    ///
    /// Two different entities at the same tick, or the same entity at two
    /// different ticks, always produce different streams.
    pub fn new(global_seed: u64, tick: WorldTick, entity_id: u128) -> Self {
        let lo = entity_id as u64;
        let hi = (entity_id >> 64) as u64;

        let mut s = global_seed
            .wrapping_mul(0x9e3779b97f4a7c15)
            .wrapping_add(tick.0.wrapping_mul(0x6c62272e07bb0142))
            .wrapping_add(lo.wrapping_mul(0x517cc1b727220a95))
            .wrapping_add(hi.wrapping_mul(0xbf58476d1ce4e5b9));

        if s == 0 { s = 1; } // xorshift64* requires non-zero state
        Self { state: s }
    }

    /// Next `u64` in the sequence (xorshift64*).
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform float in `[0.0, 1.0)`.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform integer in `[min, max)`.
    pub fn next_range(&mut self, min: i64, max: i64) -> i64 {
        assert!(min < max, "DeterministicRng::next_range: min must be < max");
        let range = (max - min) as u64;
        min + (self.next_u64() % range) as i64
    }

    /// Fisher-Yates in-place shuffle (deterministic).
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() as usize) % (i + 1);
            slice.swap(i, j);
        }
    }

    /// Returns `true` with probability `p ∈ [0.0, 1.0]`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p.clamp(0.0, 1.0)
    }
}
