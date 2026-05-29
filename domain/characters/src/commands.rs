//! Character domain commands.
//!
//! Commands express intent — they are validated by `CharacterCommandHandler`
//! (see `reducer.rs`) and, if valid, produce `CharacterEvent`s.
//!
//! **All commands are handled by `Character::handle` (the `AggregateRoot` impl).**

use events::ConversationOutcome;
use types::{CharacterId, FactionId, LocationId};

use crate::{goals::Goal, memory::Episode};

/// Every player or system intention targeting a `Character`.
#[derive(Debug, Clone)]
pub enum CharacterCommand {
    // ── Movement ──────────────────────────────────────────────────────────────
    /// Move the character to a new location.
    Move { to: LocationId },

    // ── Goals ─────────────────────────────────────────────────────────────────
    /// Push a goal onto the character's goal stack.
    AssignGoal { goal: Goal },
    /// Mark the active goal as completed.
    CompleteActiveGoal,
    /// Abandon a specific goal, recording a reason.
    AbandonGoal { goal_id: types::GoalId, reason: String },

    // ── Social ────────────────────────────────────────────────────────────────
    /// Open a conversation with another character.
    StartConversation { with: CharacterId },
    /// Close an in-progress conversation and record the outcome.
    EndConversation {
        with:    CharacterId,
        outcome: ConversationOutcome,
    },

    // ── Memory ────────────────────────────────────────────────────────────────
    /// Record a new memory episode.
    RecordEpisode {
        episode:     Episode,
        /// Hard cap on retained episodes; oldest/weakest are dropped first.
        max_memory:  usize,
    },
    /// Trigger deterministic memory decay at the current tick.
    ApplyDecay,

    // ── Factions ──────────────────────────────────────────────────────────────
    JoinFaction  { faction_id: FactionId },
    LeaveFaction { faction_id: FactionId, reason: String },
}
