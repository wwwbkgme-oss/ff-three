//! Deterministic PRNG for domain use — canonical forge-core implementation.
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
//! ## Algorithm (canonical — CONFORMANCE_VERSION 1)
//!
//! Seeding: **BLAKE3** over `(global_seed_le64 ‖ tick_le64 ‖ entity_id bytes)`.
//! Output:  **Xorshift64\*** with shifts **(13, 7, 17)** and multiplier
//!           `0x9e37_79b9_7f4a_7c15`.
//!
//! ## Migration note
//!
//! Previous implementation used XOR-multiplication seeding (weak avalanche)
//! and Xorshift64* with shifts (12, 25, 27) and multiplier `0x2545_F491_4F6C_DD1D`.
//! Updated to match forge-core canonical (CONFORMANCE_VERSION 0 → 1).

use crate::time::WorldTick;

/// Deterministic pseudo-random number generator seeded via BLAKE3.
///
/// Semantically identical to `forge-core::DeterministicRng`.
/// All federated repos must produce the same output for the same inputs
/// when using equal seeds.
#[derive(Debug, Clone)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// Canonical constructor for 128-bit entity IDs (e.g. UUID as u128).
    ///
    /// Derives the initial Xorshift64* state via BLAKE3:
    ///   `BLAKE3(global_seed_le64 ‖ tick_le64 ‖ entity_lo_le64 ‖ entity_hi_le64)`
    pub fn new(global_seed: u64, tick: WorldTick, entity_id: u128) -> Self {
        let lo = entity_id as u64;
        let hi = (entity_id >> 64) as u64;
        let mut hasher = blake3::Hasher::new();
        hasher.update(&global_seed.to_le_bytes());
        hasher.update(&tick.0.to_le_bytes());
        hasher.update(&lo.to_le_bytes());
        hasher.update(&hi.to_le_bytes());
        let hash = hasher.finalize();
        let seed = u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap());
        Self {
            state: if seed == 0 {
                0xcafe_babe_dead_beef
            } else {
                seed
            },
        }
    }

    /// Canonical constructor matching forge-core's `from_context(g, t, e: u64)`.
    /// Use when entity_id fits in 64 bits.
    pub fn from_context(global_seed: u64, tick: u64, entity_id: u64) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&global_seed.to_le_bytes());
        hasher.update(&tick.to_le_bytes());
        hasher.update(&entity_id.to_le_bytes());
        let hash = hasher.finalize();
        let seed = u64::from_le_bytes(hash.as_bytes()[..8].try_into().unwrap());
        Self {
            state: if seed == 0 {
                0xcafe_babe_dead_beef
            } else {
                seed
            },
        }
    }

    /// Direct seed — for tests and deterministic fixtures only.
    pub fn from_seed(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xcafe_babe_dead_beef
            } else {
                seed
            },
        }
    }

    // ── Output — canonical Xorshift64* (shifts 13, 7, 17) ───────────────────

    /// Next pseudo-random `u64`.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x.wrapping_mul(0x9e37_79b9_7f4a_7c15)
    }

    /// Uniform float in `[0.0, 1.0)`.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform `u64` in `[0, bound)` — rejection sampling (unbiased).
    pub fn next_bounded(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        let threshold = u64::MAX - (u64::MAX % bound);
        loop {
            let v = self.next_u64();
            if v < threshold {
                return v % bound;
            }
        }
    }

    /// Uniform integer in `[min, max)`.
    pub fn next_range(&mut self, min: i64, max: i64) -> i64 {
        assert!(min < max, "DeterministicRng::next_range: min must be < max");
        min + self.next_bounded((max - min) as u64) as i64
    }

    /// Fisher-Yates in-place shuffle (deterministic).
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = self.next_bounded(i as u64 + 1) as usize;
            slice.swap(i, j);
        }
    }

    /// Returns `true` with probability `p ∈ [0.0, 1.0]`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.next_f64() < p.clamp(0.0, 1.0)
    }
}

// ── Conformance tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_same_output() {
        let mut a = DeterministicRng::new(1, WorldTick(1), 1);
        let mut b = DeterministicRng::new(1, WorldTick(1), 1);
        assert_eq!(
            a.next_u64(),
            b.next_u64(),
            "same inputs must yield same output"
        );
    }

    #[test]
    fn zero_inputs_nonzero_output() {
        let mut rng = DeterministicRng::new(0, WorldTick(0), 0);
        assert_ne!(rng.next_u64(), 0);
    }

    #[test]
    fn entity_id_changes_output() {
        let a = DeterministicRng::new(1, WorldTick(1), 1).next_u64_take1();
        let b = DeterministicRng::new(1, WorldTick(1), 2).next_u64_take1();
        assert_ne!(a, b, "different entity_id must produce different output");
    }

    #[test]
    fn tick_changes_output() {
        let a = DeterministicRng::new(1, WorldTick(1), 1).next_u64_take1();
        let b = DeterministicRng::new(1, WorldTick(2), 1).next_u64_take1();
        assert_ne!(a, b);
    }

    #[test]
    fn blake3_avalanche() {
        let a: Vec<u64> = (0..8)
            .scan(
                DeterministicRng::new(1000, WorldTick(1000), 1000),
                |r, _| Some(r.next_u64()),
            )
            .collect();
        let b: Vec<u64> = (0..8)
            .scan(
                DeterministicRng::new(1000, WorldTick(1000), 1001),
                |r, _| Some(r.next_u64()),
            )
            .collect();
        let bits: u32 = a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x ^ y).count_ones())
            .sum();
        assert!(bits as f64 / (8 * 64) as f64 >= 0.25, "avalanche < 25%");
    }

    #[test]
    fn next_bounded_in_range() {
        let mut rng = DeterministicRng::from_seed(42);
        for _ in 0..10_000 {
            assert!(rng.next_bounded(100) < 100);
        }
    }

    #[test]
    fn from_context_matches_forge_core_interface() {
        // from_context(u64, u64, u64) must produce same result as forge-core.
        let mut a = DeterministicRng::from_context(7, 3, 11);
        let mut b = DeterministicRng::from_context(7, 3, 11);
        assert_eq!(a.next_u64(), b.next_u64());
        let mut c = DeterministicRng::from_context(7, 3, 12);
        assert_ne!(a.next_u64(), c.next_u64());
    }
}

impl DeterministicRng {
    #[cfg(test)]
    fn next_u64_take1(&mut self) -> u64 {
        self.next_u64()
    }
}
