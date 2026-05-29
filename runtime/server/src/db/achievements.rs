use sqlx::PgPool;
use types::{Achievement, AchievementType, Certification};
use uuid::Uuid;
use crate::error::{DbResult, not_found, unique_err};

pub async fn award(
    pool: &PgPool, sid: Uuid, atype: AchievementType,
    title: &str, desc: &str, xp: i32,
) -> DbResult<Achievement> {
    Ok(sqlx::query_as::<_, Achievement>(
        "INSERT INTO achievements(student_id,achievement_type,title,description,xp_reward) \
         VALUES($1,$2,$3,$4,$5) RETURNING *"
    ).bind(sid).bind(atype).bind(title).bind(desc).bind(xp)
     .fetch_one(pool).await?)
}

pub async fn list_for_student(pool: &PgPool, sid: Uuid) -> DbResult<Vec<Achievement>> {
    Ok(sqlx::query_as::<_, Achievement>(
        "SELECT * FROM achievements WHERE student_id=$1 ORDER BY earned_at DESC"
    ).bind(sid).fetch_all(pool).await?)
}

pub async fn certify(
    pool: &PgPool, sid: Uuid, path: &str,
    cred_id: &str, world_seed: &str, reviews: &serde_json::Value,
) -> DbResult<Certification> {
    unique_err(
        sqlx::query_as::<_, Certification>(
            "INSERT INTO certifications(student_id,path,credential_id,world_seed,mentor_reviews) \
             VALUES($1,$2,$3,$4,$5) RETURNING *"
        ).bind(sid).bind(path).bind(cred_id).bind(world_seed).bind(reviews)
         .fetch_one(pool).await,
        "Certification for this path already exists",
    )
}

pub async fn get_cert(pool: &PgPool, sid: Uuid, path: &str) -> DbResult<Certification> {
    not_found(
        sqlx::query_as::<_, Certification>(
            "SELECT * FROM certifications WHERE student_id=$1 AND path=$2"
        ).bind(sid).bind(path).fetch_one(pool).await,
        "Certification", path,
    )
}

pub async fn list_certs(pool: &PgPool, sid: Uuid) -> DbResult<Vec<Certification>> {
    Ok(sqlx::query_as::<_, Certification>(
        "SELECT * FROM certifications WHERE student_id=$1 ORDER BY issued_at DESC"
    ).bind(sid).fetch_all(pool).await?)
}
