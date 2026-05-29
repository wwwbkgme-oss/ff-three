use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "achievement_type", rename_all = "snake_case")]
pub enum AchievementType {
    QuestCompleted, BiomeUnlocked, StructureBuilt, PeerTeachingSession,
    LevelUp, CertificationEarned, FirstBlood, Perfectionist, Collaborator,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Achievement {
    pub id: Uuid,
    pub student_id: Uuid,
    pub achievement_type: AchievementType,
    pub title: String,
    pub description: String,
    pub xp_reward: i32,
    pub earned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Certification {
    pub id: Uuid,
    pub student_id: Uuid,
    pub path: String,
    pub credential_id: String,
    pub world_seed: String,
    pub mentor_reviews: serde_json::Value,
    pub issued_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CertifyRequest {
    pub student_id: Uuid,
    pub path: String,
}
