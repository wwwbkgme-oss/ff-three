//! Runtime configuration — read from environment variables at startup.
//!
//! ## Required
//!
//! | Variable       | Example                                  | Description          |
//! |----------------|------------------------------------------|----------------------|
//! | `DATABASE_URL` | `postgres://user:pass@localhost/forge`   | PostgreSQL DSN       |
//! | `REDIS_URL`    | `redis://localhost:6379`                 | Redis DSN            |
//! | `JWT_SECRET`   | 64-byte hex string                       | Token signing secret |
//!
//! ## LLM providers — free, set at least one
//!
//! `FreeClient` (in `runtime/drivers/llm`) auto-detects configured providers:
//!
//! | Variable             | Provider     | Free tier                      |
//! |----------------------|--------------|--------------------------------|
//! | `GROQ_API_KEY`       | Groq Cloud   | 14 400 req/day, <100 ms TTFT   |
//! | `SAMBANOVA_API_KEY`  | SambaNova    | 20–480 RPM, no credit card     |
//! | `LLM7_API_KEY`       | LLM7.io      | 100 req/hr, free token         |
//! | `OPENROUTER_API_KEY` | OpenRouter   | `:free` models, $0/token       |
//! | `NVIDIA_API_KEY`     | NVIDIA NIM   | 1 000 req/month credits        |
//! | `OLLAMA_HOST`        | Ollama local | always free, `ollama serve`    |

use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host:                  String,
    pub port:                  u16,
    pub database_url:          String,
    pub redis_url:             String,
    pub jwt_secret:            String,
    pub sandbox_timeout_secs:  u64,
    pub sandbox_max_memory_mb: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("APP_PORT").unwrap_or_else(|_| "8080".into())
                .parse().context("APP_PORT must be a valid port number")?,
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL is required (postgres://user:pass@host/db)")?,
            redis_url: env::var("REDIS_URL")
                .context("REDIS_URL is required (redis://host:port)")?,
            jwt_secret: env::var("JWT_SECRET")
                .context("JWT_SECRET is required (generate with: openssl rand -hex 64)")?,
            sandbox_timeout_secs:  env::var("SANDBOX_TIMEOUT_SECS")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(30),
            sandbox_max_memory_mb: env::var("SANDBOX_MAX_MEMORY_MB")
                .ok().and_then(|v| v.parse().ok()).unwrap_or(128),
        })
    }
}
