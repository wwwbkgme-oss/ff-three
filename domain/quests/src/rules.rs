//! Quest business constants – single source for all thresholds and formulae.

/// Normalised score (0.0–1.0) required to pass a quest.
pub const PASS_THRESHOLD: f64 = 0.70;

/// Maximum attempts a student may make on a single quest.
pub const MAX_ATTEMPTS: i32 = 10;

/// XP awarded on quest completion, scaled by difficulty.
pub fn xp_for_difficulty(difficulty: i32) -> i32 {
    difficulty.max(1) * 10
}
