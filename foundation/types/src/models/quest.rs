use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quest_type", rename_all = "snake_case")]
pub enum QuestType { Exploration, Construction, Research, Teaching, Combat }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "quest_status", rename_all = "snake_case")]
pub enum QuestStatus { Available, InProgress, Completed, Failed, Locked }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub description: String,
    pub input: String,
    pub expected_output: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Quest {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub quest_type: QuestType,
    pub difficulty: i32,
    pub xp_reward: i32,
    pub biome_id: Uuid,
    pub requirements: Vec<String>,
    pub test_cases: serde_json::Value,
    pub status: QuestStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StudentQuest {
    pub id: Uuid,
    pub student_id: Uuid,
    pub quest_id: Uuid,
    pub status: QuestStatus,
    pub attempts: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateQuestRequest {
    pub goal: String,
    pub biome_id: Option<Uuid>,
    pub difficulty: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CompleteQuestRequest {
    pub student_id: Uuid,
    pub solution: String,
    pub language: Option<String>,
    pub notes: Option<String>,
}
