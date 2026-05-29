//! Shared application state injected into every Axum handler.

use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use agents::Orchestrator;
use drivers::FreeClient;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db:           PgPool,
    pub redis:        redis::Client,
    pub config:       Arc<AppConfig>,
    /// Pure domain orchestrator – selects strategies, builds prompts.
    pub orchestrator: Arc<Orchestrator>,
    /// Free-tier LLM client — active when any provider env var is set.
    /// Chain: Groq → SambaNova → LLM7 → OpenRouter → NVIDIA NIM → Ollama.
    pub llm:          Option<Arc<FreeClient>>,
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

        // ── LLM client (free-tier auto-detection) ─────────────────────────────
        let free = FreeClient::from_env();
        let llm = if free.has_providers() {
            tracing::info!(
                "LLM: {} provider(s) active — {}",
                free.providers().len(),
                free.providers().iter().map(|p| p.kind.to_string()).collect::<Vec<_>>().join(" → ")
            );
            Some(Arc::new(free))
        } else {
            tracing::info!("LLM disabled — set GROQ_API_KEY, SAMBANOVA_API_KEY, LLM7_API_KEY, OPENROUTER_API_KEY, or NVIDIA_API_KEY to enable");
            None
        };

        Ok(Self { db, redis, config: Arc::new(config.clone()), orchestrator, llm })
    }
}
