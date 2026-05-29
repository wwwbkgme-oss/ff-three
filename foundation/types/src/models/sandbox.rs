use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "sandbox_status", rename_all = "snake_case")]
pub enum SandboxStatus {
    Pending, Running, Completed, Failed, Timeout, SecurityViolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "programming_language", rename_all = "snake_case")]
pub enum ProgrammingLanguage { Rust, Python, JavaScript, TypeScript, Go, Java, Cpp }

impl ProgrammingLanguage {
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Rust       => "rs",  Self::Python     => "py",
            Self::JavaScript => "js",  Self::TypeScript => "ts",
            Self::Go         => "go",  Self::Java       => "java",
            Self::Cpp        => "cpp",
        }
    }
    pub fn docker_image(&self) -> &'static str {
        match self {
            Self::Rust       => "rust:1.76-slim",
            Self::Python     => "python:3.12-slim",
            Self::JavaScript | Self::TypeScript => "node:22-slim",
            Self::Go         => "golang:1.22-slim",
            Self::Java       => "eclipse-temurin:21-jre-jammy",
            Self::Cpp        => "gcc:13-bookworm",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScan {
    pub passed: bool,
    pub threats_detected: Vec<String>,
    /// "none" | "low" | "medium" | "high"
    pub risk_level: String,
    pub scanned_at: DateTime<Utc>,
}

impl SecurityScan {
    pub fn clean() -> Self {
        Self { passed: true, threats_detected: vec![], risk_level: "none".into(), scanned_at: Utc::now() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SandboxRun {
    pub id: Uuid,
    pub student_id: Uuid,
    pub quest_id: Option<Uuid>,
    pub language: ProgrammingLanguage,
    pub code: String,
    pub status: SandboxStatus,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
    pub execution_time_ms: Option<i64>,
    pub memory_used_kb: Option<i64>,
    pub security_scan: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct PracticeRequest {
    pub student_id: Uuid,
    pub language: ProgrammingLanguage,
    pub quest_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitSolutionRequest {
    pub student_id: Uuid,
    pub quest_id: Uuid,
    pub language: ProgrammingLanguage,
    pub code: String,
}
