//! All Academy domain events.
//!
//! Events are immutable facts about what happened.
//! The same event sequence must always produce the same state (replay-safe).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use types::{AchievementType, BiomeState};

/// Every state change in the Academy is expressed as one of these variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcademyEvent {
    // ── Student lifecycle ─────────────────────────────────────────────────────
    StudentEnrolled {
        student_id: Uuid,
        username: String,
        email: String,
        initial_biome_slug: String,
        timestamp: DateTime<Utc>,
    },
    StudentGoalsUpdated {
        student_id: Uuid,
        goals: Vec<String>,
        timestamp: DateTime<Utc>,
    },

    // ── Quest progression ─────────────────────────────────────────────────────
    QuestStarted {
        student_id: Uuid,
        quest_id: Uuid,
        attempt: i32,
        timestamp: DateTime<Utc>,
    },
    QuestCompleted {
        student_id: Uuid,
        quest_id: Uuid,
        score: f64,
        xp_awarded: i32,
        timestamp: DateTime<Utc>,
    },
    QuestFailed {
        student_id: Uuid,
        quest_id: Uuid,
        score: f64,
        attempt: i32,
        timestamp: DateTime<Utc>,
    },

    // ── Progression ───────────────────────────────────────────────────────────
    XpGained {
        student_id: Uuid,
        amount: i32,
        new_total: i32,
        timestamp: DateTime<Utc>,
    },
    LevelUp {
        student_id: Uuid,
        new_level: i32,
        timestamp: DateTime<Utc>,
    },

    // ── World exploration ─────────────────────────────────────────────────────
    BiomeEntered {
        student_id: Uuid,
        biome_id: Uuid,
        biome_slug: String,
        timestamp: DateTime<Utc>,
    },
    BiomeStateChanged {
        biome_id: Uuid,
        new_state: BiomeState,
        avg_score: f64,
        timestamp: DateTime<Utc>,
    },

    // ── Knowledge graph ───────────────────────────────────────────────────────
    ConceptMasteryUpdated {
        student_id: Uuid,
        concept: String,
        mastery: f64,
        timestamp: DateTime<Utc>,
    },

    // ── Collaboration ─────────────────────────────────────────────────────────
    GroupCreated {
        group_id: Uuid,
        name: String,
        goal: String,
        timestamp: DateTime<Utc>,
    },
    GroupJoined {
        group_id: Uuid,
        student_id: Uuid,
        timestamp: DateTime<Utc>,
    },

    // ── Achievements & certifications ─────────────────────────────────────────
    AchievementEarned {
        student_id: Uuid,
        achievement_type: AchievementType,
        title: String,
        xp_reward: i32,
        timestamp: DateTime<Utc>,
    },
    CertificationIssued {
        student_id: Uuid,
        path: String,
        credential_id: String,
        world_seed: String,
        timestamp: DateTime<Utc>,
    },
}

impl AcademyEvent {
    /// Returns the UTC timestamp carried by any event variant.
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::StudentEnrolled      { timestamp, .. } => *timestamp,
            Self::StudentGoalsUpdated  { timestamp, .. } => *timestamp,
            Self::QuestStarted         { timestamp, .. } => *timestamp,
            Self::QuestCompleted       { timestamp, .. } => *timestamp,
            Self::QuestFailed          { timestamp, .. } => *timestamp,
            Self::XpGained             { timestamp, .. } => *timestamp,
            Self::LevelUp              { timestamp, .. } => *timestamp,
            Self::BiomeEntered         { timestamp, .. } => *timestamp,
            Self::BiomeStateChanged    { timestamp, .. } => *timestamp,
            Self::ConceptMasteryUpdated{ timestamp, .. } => *timestamp,
            Self::GroupCreated         { timestamp, .. } => *timestamp,
            Self::GroupJoined          { timestamp, .. } => *timestamp,
            Self::AchievementEarned    { timestamp, .. } => *timestamp,
            Self::CertificationIssued  { timestamp, .. } => *timestamp,
        }
    }
}
