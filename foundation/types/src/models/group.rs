use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "group_status", rename_all = "snake_case")]
pub enum GroupStatus { Active, Completed, Disbanded }

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StudyGroup {
    pub id: Uuid,
    pub name: String,
    pub goal: String,
    pub biome_id: Option<Uuid>,
    pub progress: f64,
    pub status: GroupStatus,
    pub max_members: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub student_id: Uuid,
    pub role: String,
    pub contribution: f64,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub goal: String,
    pub biome_id: Option<Uuid>,
    pub max_members: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMemberDetail {
    pub student_id: Uuid,
    pub username: String,
    pub role: String,
    pub contribution: f64,
}

#[derive(Debug, Serialize)]
pub struct GroupProgressResponse {
    pub group: StudyGroup,
    pub members: Vec<GroupMemberDetail>,
    pub collaborative_structure: Option<String>,
    pub progress_pct: f64,
}
