//! Shared application state injected into every Axum handler.

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use agents::Orchestrator;

use crate::{config::AppConfig, llm::LlmClient};

#[derive(Clone)]
pub struct AppState {
    pub db:           PgPool,
    pub redis:        redis::Client,
    pub config:       Arc<AppConfig>,
    /// Pure domain orchestrator – selects strategies, builds prompts.
    pub orchestrator: Arc<Orchestrator>,
    /// Optional LLM client – present when OPENAI_API_KEY is set.
    pub llm:          Option<Arc<LlmClient>>,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> anyhow::Result<Self> {
        // ── PostgreSQL ────────────────────────────────────────────────────────
        let db = PgPoolOptions::new()
            .max_connections(20).min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(&config.database_url).await?;
        tracing::info!("Connected to PostgreSQL");

        // ── Redis ─────────────────────────────────────────────────────────────
        let redis = redis::Client::open(config.redis_url.as_str())?;
        {
            let mut con = redis.get_multiplexed_async_connection().await?;
            let _: () = redis::cmd("PING").query_async(&mut con).await?;
        }
        tracing::info!("Connected to Redis");

        // ── Domain orchestrator ───────────────────────────────────────────────
        let orchestrator = Arc::new(Orchestrator::new());

        // ── LLM client (optional) ─────────────────────────────────────────────
        let llm = config.openai_api_key.as_ref().map(|key| {
            tracing::info!("LLM backend enabled ({})", config.openai_model);
            Arc::new(LlmClient::new(key.clone(), config.openai_model.clone()))
        });
        if llm.is_none() {
            tracing::info!("LLM disabled – using deterministic agent stubs");
        }

        Ok(Self { db, redis, config: Arc::new(config.clone()), orchestrator, llm })
    }
}
