//! Runtime configuration read from environment variables.

use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub sandbox_timeout_secs: u64,
    pub sandbox_max_memory_mb: u64,
    pub openai_api_key: Option<String>,
    pub openai_model: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("APP_PORT").unwrap_or_else(|_| "8080".into())
                .parse().context("APP_PORT must be a valid port")?,
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL required (postgres://user:pass@host/db)")?,
            redis_url: env::var("REDIS_URL")
                .context("REDIS_URL required (redis://host:port)")?,
            jwt_secret: env::var("JWT_SECRET")
                .context("JWT_SECRET required")?,
            sandbox_timeout_secs: env::var("SANDBOX_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".into()).parse().unwrap_or(30),
            sandbox_max_memory_mb: env::var("SANDBOX_MAX_MEMORY_MB")
                .unwrap_or_else(|_| "128".into()).parse().unwrap_or(128),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            openai_model: env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".into()),
        })
    }

    pub fn llm_enabled(&self) -> bool { self.openai_api_key.is_some() }
}
