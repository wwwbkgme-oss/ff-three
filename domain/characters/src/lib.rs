//! `characters` – domain crate for the character simulation system.
//!
//! Implements deterministic state for Players, NPCs, Agents, and Companions.
//! No I/O. All state changes expressed as `CharacterEvent` and applied
//! through `CharacterReducer`.
//!
//! Architecture:
//!   Command → CharacterCommandHandler → [CharacterEvent]
//!   [CharacterEvent] → CharacterReducer::apply → Character

pub mod character;
pub mod commands;
pub mod errors;
pub mod goals;
pub mod memory;
pub mod planner;
pub mod reducer;
pub mod relationships;
pub mod schedule;
pub mod stats;
pub mod tick;

// ── Public API ────────────────────────────────────────────────────────────────
pub use character::{Activity, Character};
pub use commands::CharacterCommand;
pub use errors::CharacterError;
pub use goals::{Condition, Goal, GoalStack, GoalType};
pub use memory::{Belief, Episode, Memory, DECAY_RATE_PER_TICK, FORGET_THRESHOLD};
pub use planner::Planner;
pub use reducer::CharacterReducer;
pub use relationships::{Relationship, RelationshipGraph};
pub use schedule::{Schedule, ScheduledActivity, TimeSlot};
pub use stats::{Mood, Stats};
pub use tick::TickEngine;
