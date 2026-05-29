//! Deterministic simulation time.
//!
//! `WorldTick` replaces all wall-clock usage inside `domain/`.
//! Runtime increments the tick; domain crates read it only via `CommandContext`.
//!
//! Rule: `chrono::Utc::now()` is **forbidden** in `domain/` and `foundation/`.

use serde::{Deserialize, Serialize};

/// Monotonically increasing simulation counter.
/// One tick = one logical step; duration is runtime-defined.
/// Stored as `BIGINT` in the DB via manual `i64` conversion; no array usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         Serialize, Deserialize, Default)]
pub struct WorldTick(pub u64);

impl WorldTick {
    pub const ZERO: Self = Self(0);

    pub fn advance(self, delta: u64) -> Self { Self(self.0 + delta) }
    pub fn elapsed_since(self, earlier: Self) -> u64 {
        self.0.saturating_sub(earlier.0)
    }
}

impl std::fmt::Display for WorldTick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "tick({})", self.0)
    }
}

/// Number of ticks that constitute one in-world day.
/// Configurable at runtime; 2400 = 1 tick per 36 real seconds at 1 Hz.
pub const DAY_LENGTH_TICKS: u64 = 2_400;

/// Tick-within-day: `current_tick % DAY_LENGTH_TICKS`.
pub fn tick_of_day(tick: WorldTick) -> u64 { tick.0 % DAY_LENGTH_TICKS }

/// Convert a fraction of the day (0.0..1.0) to a day-relative tick offset.
pub fn day_fraction_to_tick(fraction: f64) -> u64 {
    (fraction.clamp(0.0, 1.0) * DAY_LENGTH_TICKS as f64) as u64
}
