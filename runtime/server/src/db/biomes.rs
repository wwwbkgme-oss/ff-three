use sqlx::PgPool;
use types::{Biome, BiomeState};
use uuid::Uuid;
use crate::error::{DbResult, not_found};

pub async fn list(pool: &PgPool) -> DbResult<Vec<Biome>> {
    Ok(sqlx::query_as::<_, Biome>("SELECT * FROM biomes ORDER BY name")
        .fetch_all(pool).await?)
}

pub async fn get(pool: &PgPool, id: Uuid) -> DbResult<Biome> {
    not_found(
        sqlx::query_as::<_, Biome>("SELECT * FROM biomes WHERE id=$1")
            .bind(id).fetch_one(pool).await,
        "Biome", &id.to_string(),
    )
}

pub async fn get_by_slug(pool: &PgPool, slug: &str) -> DbResult<Biome> {
    not_found(
        sqlx::query_as::<_, Biome>("SELECT * FROM biomes WHERE slug=$1")
            .bind(slug).fetch_one(pool).await,
        "Biome", slug,
    )
}

pub async fn set_state(pool: &PgPool, id: Uuid, state: BiomeState) -> DbResult<Biome> {
    not_found(
        sqlx::query_as::<_, Biome>("UPDATE biomes SET state=$2 WHERE id=$1 RETURNING *")
            .bind(id).bind(state).fetch_one(pool).await,
        "Biome", &id.to_string(),
    )
}

pub async fn incr_active(pool: &PgPool, id: Uuid) -> DbResult<()> {
    sqlx::query("UPDATE biomes SET active_students=active_students+1 WHERE id=$1")
        .bind(id).execute(pool).await?;
    Ok(())
}

pub async fn decr_active(pool: &PgPool, id: Uuid) -> DbResult<()> {
    sqlx::query("UPDATE biomes SET active_students=GREATEST(0,active_students-1) WHERE id=$1")
        .bind(id).execute(pool).await?;
    Ok(())
}

/// Fetch average assessment score for the last 50 quests in a biome.
pub async fn avg_score(pool: &PgPool, biome_id: Uuid) -> DbResult<Option<f64>> {
    let row: Option<(Option<f64>,)> = sqlx::query_as(
        "SELECT AVG(a.score) FROM assessments a \
         JOIN quests q ON q.id=a.quest_id WHERE q.biome_id=$1 LIMIT 50"
    ).bind(biome_id).fetch_optional(pool).await?;
    Ok(row.and_then(|(v,)| v))
}
