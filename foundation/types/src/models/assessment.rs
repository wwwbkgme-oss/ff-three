use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "assessment_type", rename_all = "snake_case")]
pub enum AssessmentType { Theory, Practice, Application, Teaching }

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Assessment {
    pub id: Uuid,
    pub student_id: Uuid,
    pub quest_id: Uuid,
    pub assessment_type: AssessmentType,
    pub submission: String,
    pub score: f64,
    pub passed: bool,
    pub feedback: String,
    pub test_results: serde_json::Value,
    pub performance_metrics: serde_json::Value,
    pub assessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_case_index: i32,
    pub passed: bool,
    pub actual_output: String,
    pub expected_output: String,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub time_complexity: Option<String>,
    pub space_complexity: Option<String>,
    pub execution_time_ms: u64,
    pub memory_used_kb: u64,
    pub code_quality_score: f64,
}
