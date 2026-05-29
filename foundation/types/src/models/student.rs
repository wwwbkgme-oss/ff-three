use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A learner enrolled in the Academy.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Student {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    /// Total XP accumulated across all quests.
    pub xp: i32,
    /// Level 1–10, derived from XP by [`level_for_xp`].
    pub level: i32,
    pub current_biome_id: Option<Uuid>,
    pub enrolled_at: DateTime<Utc>,
    pub goals: Vec<String>,
    /// Knowledge graph stored as JSONB; use [`KnowledgeGraph`] to interpret.
    pub knowledge_map: serde_json::Value,
    pub mentor_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct EnrollRequest {
    pub username: String,
    pub email: String,
    pub topic: Option<String>,
    pub goals: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct EnrollResponse {
    pub student_id: String,
    pub message: String,
    pub initial_biome: String,
    pub mentor_assigned: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGoalsRequest {
    pub goals: Vec<String>,
}

/// Deterministic XP → level mapping.
/// Pure function; safe to call in foundation layer.
pub fn level_for_xp(xp: i32) -> i32 {
    match xp {
        x if x < 100   => 1,
        x if x < 300   => 2,
        x if x < 600   => 3,
        x if x < 1_000 => 4,
        x if x < 1_500 => 5,
        x if x < 2_500 => 6,
        x if x < 4_000 => 7,
        x if x < 6_000 => 8,
        x if x < 9_000 => 9,
        _              => 10,
    }
}
