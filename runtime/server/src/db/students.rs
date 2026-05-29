use sqlx::PgPool;
use types::{EnrollRequest, KnowledgeGraph, Student, level_for_xp};
use uuid::Uuid;
use crate::error::{DbResult, not_found, unique_err};

pub async fn create(pool: &PgPool, req: &EnrollRequest) -> DbResult<Student> {
    let goals = req.goals.clone().unwrap_or_default();
    let kg = KnowledgeGraph::new(Uuid::new_v4()).to_json();
    unique_err(
        sqlx::query_as::<_, Student>(
            "INSERT INTO students (username,email,goals,knowledge_map) VALUES($1,$2,$3,$4) RETURNING *"
        ).bind(&req.username).bind(&req.email).bind(&goals).bind(&kg)
        .fetch_one(pool).await,
        "Username or email already exists",
    )
}

pub async fn get(pool: &PgPool, id: Uuid) -> DbResult<Student> {
    not_found(
        sqlx::query_as::<_, Student>("SELECT * FROM students WHERE id=$1")
            .bind(id).fetch_one(pool).await,
        "Student", &id.to_string(),
    )
}

pub async fn list(pool: &PgPool) -> DbResult<Vec<Student>> {
    Ok(sqlx::query_as::<_, Student>("SELECT * FROM students ORDER BY level DESC,xp DESC")
        .fetch_all(pool).await?)
}

pub async fn set_xp_level(pool: &PgPool, id: Uuid, new_xp: i32) -> DbResult<Student> {
    let level = level_for_xp(new_xp);
    not_found(
        sqlx::query_as::<_, Student>(
            "UPDATE students SET xp=$2,level=$3 WHERE id=$1 RETURNING *"
        ).bind(id).bind(new_xp).bind(level).fetch_one(pool).await,
        "Student", &id.to_string(),
    )
}

pub async fn add_xp(pool: &PgPool, id: Uuid, delta: i32) -> DbResult<Student> {
    let row: (i32,) = sqlx::query_as("UPDATE students SET xp=xp+$2 WHERE id=$1 RETURNING xp")
        .bind(id).bind(delta).fetch_one(pool).await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => crate::error::DbError::NotFound { entity: "Student", id: id.to_string() },
            other => crate::error::DbError::Sqlx(other),
        })?;
    set_xp_level(pool, id, row.0).await
}

pub async fn update_goals(pool: &PgPool, id: Uuid, goals: Vec<String>) -> DbResult<Student> {
    not_found(
        sqlx::query_as::<_, Student>("UPDATE students SET goals=$2 WHERE id=$1 RETURNING *")
            .bind(id).bind(&goals).fetch_one(pool).await,
        "Student", &id.to_string(),
    )
}

pub async fn set_biome(pool: &PgPool, id: Uuid, biome_id: Uuid) -> DbResult<Student> {
    not_found(
        sqlx::query_as::<_, Student>(
            "UPDATE students SET current_biome_id=$2 WHERE id=$1 RETURNING *"
        ).bind(id).bind(biome_id).fetch_one(pool).await,
        "Student", &id.to_string(),
    )
}

pub async fn update_knowledge_map(pool: &PgPool, id: Uuid, km: &serde_json::Value) -> DbResult<Student> {
    not_found(
        sqlx::query_as::<_, Student>(
            "UPDATE students SET knowledge_map=$2 WHERE id=$1 RETURNING *"
        ).bind(id).bind(km).fetch_one(pool).await,
        "Student", &id.to_string(),
    )
}
