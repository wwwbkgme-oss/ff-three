//! Database repositories – PostgreSQL I/O via SQLx.
//!
//! BKG runtime layer: all DB access lives here.
//! Repositories are pure CRUD; domain logic stays in domain/ crates.

pub mod achievements;
pub mod assessments;
pub mod biomes;
pub mod groups;
pub mod quests;
pub mod sandbox;
pub mod students;

use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Migration failed: {e}"))?;
    tracing::info!("Database migrations applied");
    Ok(())
}
