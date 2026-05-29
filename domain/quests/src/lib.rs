//! `quests` domain – quest lifecycle, student progress, achievement rules.
//!
//! Pure business logic. No I/O, no database, no network.
//! All state changes are expressed as [`AcademyEvent`] values.

pub mod reducer;
pub mod rules;
pub mod state;

pub use reducer::QuestCommandHandler;
pub use rules::{xp_for_difficulty, MAX_ATTEMPTS, PASS_THRESHOLD};
pub use state::StudentProgress;
