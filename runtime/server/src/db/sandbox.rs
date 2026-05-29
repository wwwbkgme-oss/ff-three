use chrono::Utc;
use sqlx::PgPool;
use types::{ProgrammingLanguage, SandboxRun, SandboxStatus, SecurityScan};
use uuid::Uuid;
use crate::error::{DbResult, not_found};

pub async fn create(
    pool: &PgPool, sid: Uuid, qid: Option<Uuid>, lang: ProgrammingLanguage, code: String,
) -> DbResult<SandboxRun> {
    let scan = serde_json::to_value(SecurityScan::clean()).unwrap_or_default();
    Ok(sqlx::query_as::<_, SandboxRun>(
        "INSERT INTO sandbox_runs(student_id,quest_id,language,code,status,security_scan) \
         VALUES($1,$2,$3,$4,'pending',$5) RETURNING *"
    ).bind(sid).bind(qid).bind(lang).bind(&code).bind(&scan)
     .fetch_one(pool).await?)
}

pub async fn complete(
    pool: &PgPool, id: Uuid, status: SandboxStatus,
    stdout: Option<String>, stderr: Option<String>, exit_code: Option<i32>,
    exec_ms: Option<i64>, mem_kb: Option<i64>, scan: &SecurityScan,
) -> DbResult<SandboxRun> {
    let scan_json = serde_json::to_value(scan).unwrap_or_default();
    not_found(
        sqlx::query_as::<_, SandboxRun>(
            "UPDATE sandbox_runs SET status=$2,stdout=$3,stderr=$4,exit_code=$5,\
             execution_time_ms=$6,memory_used_kb=$7,security_scan=$8,completed_at=$9 \
             WHERE id=$1 RETURNING *"
        ).bind(id).bind(status).bind(stdout).bind(stderr).bind(exit_code)
         .bind(exec_ms).bind(mem_kb).bind(&scan_json).bind(Utc::now())
         .fetch_one(pool).await,
        "SandboxRun", &id.to_string(),
    )
}

pub async fn get(pool: &PgPool, id: Uuid) -> DbResult<SandboxRun> {
    not_found(
        sqlx::query_as::<_, SandboxRun>("SELECT * FROM sandbox_runs WHERE id=$1")
            .bind(id).fetch_one(pool).await,
        "SandboxRun", &id.to_string(),
    )
}
