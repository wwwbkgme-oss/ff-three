//! Character domain events.
//!
//! All mutations to `Character` state MUST be expressed as one of these
//! variants and applied through `CharacterReducer::apply`.
//!
//! **Never mutate character state directly.** Command → Event → Reducer only.

use serde::{Deserialize, Serialize};

use types::{
    CharacterId, EpisodeId, FactionId, GoalId, LocationId, WorldTick,
};

/// Every state change to a Character aggregate is expressed here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CharacterEvent {
    // ── Lifecycle ─────────────────────────────────────────────────────────────
    Created {
        id:         CharacterId,
        kind:       CharacterKind,
        name:       String,
        location:   LocationId,
        born_at:    WorldTick,
    },
    Destroyed {
        id: CharacterId,
        at: WorldTick,
    },

    // ── Movement ──────────────────────────────────────────────────────────────
    Moved {
        id:   CharacterId,
        from: LocationId,
        to:   LocationId,
        at:   WorldTick,
    },

    // ── Goals ─────────────────────────────────────────────────────────────────
    GoalAdded {
        character_id: CharacterId,
        goal:         SerializedGoal,
        at:           WorldTick,
    },
    GoalActivated {
        character_id: CharacterId,
        goal_id:      GoalId,
        at:           WorldTick,
    },
    GoalCompleted {
        character_id: CharacterId,
        goal_id:      GoalId,
        at:           WorldTick,
    },
    GoalAbandoned {
        character_id: CharacterId,
        goal_id:      GoalId,
        reason:       String,
        at:           WorldTick,
    },

    // ── Memory ────────────────────────────────────────────────────────────────
    EpisodeRecorded {
        character_id: CharacterId,
        episode:      SerializedEpisode,
    },
    /// Deterministic memory decay applied at the given tick.
    MemoryDecayApplied {
        character_id: CharacterId,
        at:           WorldTick,
    },
    EpisodeForgotten {
        character_id: CharacterId,
        episode_id:   EpisodeId,
        at:           WorldTick,
    },

    // ── Social ────────────────────────────────────────────────────────────────
    RelationshipUpdated {
        from:           CharacterId,
        to:             CharacterId,
        trust_delta:    f32,
        affinity_delta: f32,
        at:             WorldTick,
    },
    ConversationStarted {
        initiator: CharacterId,
        partner:   CharacterId,
        at:        WorldTick,
    },
    ConversationEnded {
        initiator:   CharacterId,
        partner:     CharacterId,
        outcome:     ConversationOutcome,
        at:          WorldTick,
    },

    // ── Factions ──────────────────────────────────────────────────────────────
    JoinedFaction {
        character_id: CharacterId,
        faction_id:   FactionId,
        at:           WorldTick,
    },
    LeftFaction {
        character_id: CharacterId,
        faction_id:   FactionId,
        reason:       String,
        at:           WorldTick,
    },

    // ── Mood / stats ──────────────────────────────────────────────────────────
    MoodChanged {
        character_id: CharacterId,
        new_mood:     MoodKind,
        reason:       String,
        at:           WorldTick,
    },
    StatsUpdated {
        character_id: CharacterId,
        health_delta: i32,
        energy_delta: i32,
        at:           WorldTick,
    },
}

impl CharacterEvent {
    /// Extract the WorldTick carried by this event.
    pub fn tick(&self) -> WorldTick {
        match self {
            Self::Created          { born_at, .. } => *born_at,
            Self::Destroyed        { at, .. }      => *at,
            Self::Moved            { at, .. }      => *at,
            Self::GoalAdded        { at, .. }      => *at,
            Self::GoalActivated    { at, .. }      => *at,
            Self::GoalCompleted    { at, .. }      => *at,
            Self::GoalAbandoned    { at, .. }      => *at,
            Self::MemoryDecayApplied { at, .. }    => *at,
            Self::EpisodeForgotten { at, .. }      => *at,
            Self::RelationshipUpdated { at, .. }   => *at,
            Self::ConversationStarted { at, .. }   => *at,
            Self::ConversationEnded   { at, .. }   => *at,
            Self::JoinedFaction    { at, .. }      => *at,
            Self::LeftFaction      { at, .. }      => *at,
            Self::MoodChanged      { at, .. }      => *at,
            Self::StatsUpdated     { at, .. }      => *at,
            // Events without a tick (recorded asynchronously)
            Self::EpisodeRecorded  { .. }          => WorldTick::ZERO,
        }
    }
}

// ── Supporting types (inline to avoid cross-crate coupling) ───────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CharacterKind { Player, Npc, Agent, Companion }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConversationOutcome {
    Friendly,
    Neutral,
    Hostile,
    QuestAssigned,
    InformationShared { topic: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MoodKind {
    Calm, Happy, Anxious, Angry, Sad, Fearful, Determined,
}

/// Serialized goal payload embedded in events.
/// Full `Goal` type lives in `domain/characters`; this is the foundation-layer
/// minimal representation for event log storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedGoal {
    pub id:       GoalId,
    pub kind:     String,   // display name of GoalType variant
    pub priority: i32,
    pub deadline: Option<WorldTick>,
}

/// Serialized episode payload embedded in events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedEpisode {
    pub id:           EpisodeId,
    pub summary:      String,
    pub weight:       f32,
    pub observed_at:  WorldTick,
}
