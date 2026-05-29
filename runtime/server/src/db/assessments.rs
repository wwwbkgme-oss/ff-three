use sqlx::PgPool;
use types::{Assessment, AssessmentType, PerformanceMetrics, TestResult};
use uuid::Uuid;
use crate::error::{DbResult, not_found};

pub struct CreateAssessment {
    pub student_id:   Uuid,
    pub quest_id:     Uuid,
    pub atype:        AssessmentType,
    pub submission:   String,
    pub score:        f64,
    pub passed:       bool,
    pub feedback:     String,
    pub test_results: Vec<TestResult>,
    pub metrics:      PerformanceMetrics,
}

pub async fn create(pool: &PgPool, p: CreateAssessment) -> DbResult<Assessment> {
    let tr = serde_json::to_value(&p.test_results).unwrap_or_default();
    let pm = serde_json::to_value(&p.metrics).unwrap_or_default();
    Ok(sqlx::query_as::<_, Assessment>(
        "INSERT INTO assessments(student_id,quest_id,assessment_type,submission,score,passed,feedback,test_results,performance_metrics) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING *"
    ).bind(p.student_id).bind(p.quest_id).bind(p.atype).bind(&p.submission)
     .bind(p.score).bind(p.passed).bind(&p.feedback).bind(&tr).bind(&pm)
     .fetch_one(pool).await?)
}

pub async fn get(pool: &PgPool, id: Uuid) -> DbResult<Assessment> {
    not_found(
        sqlx::query_as::<_, Assessment>("SELECT * FROM assessments WHERE id=$1")
            .bind(id).fetch_one(pool).await,
        "Assessment", &id.to_string(),
    )
}

pub async fn list_for_student(pool: &PgPool, sid: Uuid) -> DbResult<Vec<Assessment>> {
    Ok(sqlx::query_as::<_, Assessment>(
        "SELECT * FROM assessments WHERE student_id=$1 ORDER BY assessed_at DESC"
    ).bind(sid).fetch_all(pool).await?)
}
