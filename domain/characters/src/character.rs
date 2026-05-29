//! Core character aggregate state.

use serde::{Deserialize, Serialize};

use types::{CharacterId, FactionId, LocationId, WorldTick};

use crate::{
    goals::GoalStack, memory::Memory, relationships::RelationshipGraph,
    schedule::Schedule, stats::{Mood, Stats},
};

/// The complete, deterministic state of one character aggregate.
///
/// This is a pure data struct — no methods with side effects.
/// All mutations arrive as `CharacterEvent` values applied by `CharacterReducer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id:            CharacterId,
    pub name:          String,
    pub kind:          CharacterKind,
    pub stats:         Stats,
    /// What the character is doing right now.
    pub activity:      Activity,
    pub location:      LocationId,
    pub goals:         GoalStack,
    pub schedule:      Schedule,
    pub memory:        Memory,
    pub relationships: RelationshipGraph,
    pub mood:          Mood,
    pub faction:       Option<FactionId>,
    /// Deterministic birth tick – used for age and time calculations.
    pub born_at:       WorldTick,
}

impl Character {
    /// Construct an NPC with sensible defaults at the given tick and location.
    pub fn new_npc(
        id:       CharacterId,
        name:     impl Into<String>,
        location: LocationId,
        born_at:  WorldTick,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            kind: CharacterKind::Npc,
            stats: Stats::default(),
            activity: Activity::Idle,
            location,
            goals: GoalStack::default(),
            schedule: Schedule::default_npc(),
            memory: Memory::default(),
            relationships: RelationshipGraph::default(),
            mood: Mood::Calm,
            faction: None,
            born_at,
        }
    }

    /// Age of the character in ticks.
    pub fn age(&self, current_tick: WorldTick) -> u64 {
        current_tick.0.saturating_sub(self.born_at.0)
    }
}

/// Classification of the character's controller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharacterKind {
    /// Controlled by a human player.
    Player,
    /// Deterministic NPC driven by schedule + goals.
    Npc,
    /// LLM-augmented agent (goal selection assisted by runtime/agents).
    Agent,
    /// Player companion.
    Companion,
}

/// What the character is actively doing at this tick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Activity {
    Idle,
    /// Executing the goal at the top of the stack.
    ExecutingGoal(types::GoalId),
    Traveling { to: LocationId },
    Conversing { with: CharacterId },
    Working,
    Eating,
    Sleeping,
    Resting,
}
