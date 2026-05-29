//! Free-tier LLM driver — automatic provider failover.
//!
//! `FreeClient` is the single public interface for all LLM inference in
//! ForgeFabrik.  It wraps six free-tier providers and falls back
//! automatically on rate-limits and errors.
//!
//! ## Layer position
//!
//! ```text
//! domain/agents  (AgentStrategy trait — abstract, no I/O)
//!       ↑
//!       │  runtime wires concrete implementation
//!       ↓
//! runtime/drivers/llm  (FreeClient — I/O, this module)
//! ```
//!
//! Domain code never imports `FreeClient` directly.  The runtime injects it
//! through `AppState.llm` as an `Arc<FreeClient>`.
//!
//! ## Provider chain
//!
//! Auto-detected from environment variables at startup:
//!
//!   Groq → SambaNova → LLM7 → OpenRouter → NVIDIA NIM → Ollama
//!
//! Set any provider's env var to activate it.  Ollama is always appended
//! as the last-resort local fallback.

pub mod types;
pub mod catalog;
pub mod providers;

pub use types::ChatMessage;

use anyhow::Context;
use tracing::{info, instrument, warn};

use catalog::{GROQ_MODELS, LLM7_MODELS, NVIDIA_MODELS, OLLAMA_MODELS, OPENROUTER_MODELS, SAMBANOVA_MODELS};
use providers::{groq, llm7, nvidia, ollama, openrouter, sambanova};
use types::{ActiveProvider, ProviderKind};

// ── FreeClient ────────────────────────────────────────────────────────────────

/// Free-tier LLM client with automatic provider failover.
pub struct FreeClient {
    http:  reqwest::Client,
    chain: Vec<ActiveProvider>,
}

impl FreeClient {
    /// Build from environment variables.
    pub fn from_env() -> Self {
        let chain = build_chain();
        if chain.is_empty() {
            warn!("llm-driver: no providers configured — AI features disabled");
        } else {
            info!(
                providers = chain.iter().map(|p| p.kind.to_string()).collect::<Vec<_>>().join(" → "),
                "llm-driver: chain ready"
            );
        }
        Self { http: reqwest::Client::new(), chain }
    }

    pub fn has_providers(&self) -> bool { !self.chain.is_empty() }
    pub fn providers(&self) -> &[ActiveProvider] { &self.chain }

    // ── Core ──────────────────────────────────────────────────────────────────

    /// Send a chat request, falling back through the provider chain.
    #[instrument(skip(self, messages))]
    pub async fn chat(
        &self,
        messages:   Vec<ChatMessage>,
        max_tokens: u32,
    ) -> anyhow::Result<String> {
        if self.chain.is_empty() {
            anyhow::bail!("no LLM providers configured — set GROQ_API_KEY, SAMBANOVA_API_KEY, LLM7_API_KEY, OPENROUTER_API_KEY, NVIDIA_API_KEY, or run `ollama serve`");
        }

        let mut last = String::new();
        for p in &self.chain {
            match providers::openai_call(
                &self.http, p.kind, &p.base_url, &p.api_key, p.model_id, &messages, max_tokens,
            ).await {
                Ok(text) => return Ok(text),
                Err(e) if providers::is_rate_limit(&e) => {
                    warn!(provider = %p.kind, "rate-limited, trying next");
                    last = e.to_string();
                }
                Err(e) => {
                    warn!(provider = %p.kind, error = %e, "error, trying next");
                    last = e.to_string();
                }
            }
        }
        anyhow::bail!("all LLM providers failed (last: {last})")
    }

    // ── Domain helpers (drop-in for runtime/server) ───────────────────────────

    pub async fn run_quest_generation(&self, prompt: &str) -> anyhow::Result<(String, String)> {
        let raw = self.chat(vec![
            ChatMessage::system(
                "You are a game designer for an educational platform. \
                 Respond with valid JSON: {\"title\": \"...\", \"description\": \"...\"}"
            ),
            ChatMessage::user(prompt),
        ], 300).await?;
        let v: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("LLM returned non-JSON: {raw}"))?;
        Ok((
            v["title"].as_str().unwrap_or("Learning Quest").to_string(),
            v["description"].as_str().unwrap_or("Complete this quest.").to_string(),
        ))
    }

    pub async fn run_evaluation(&self, prompt: &str) -> anyhow::Result<(f64, String)> {
        let raw = self.chat(vec![
            ChatMessage::system(
                "You are a code review expert. \
                 Respond with valid JSON: {\"score\": 0.0-1.0, \"feedback\": \"...\"}"
            ),
            ChatMessage::user(prompt),
        ], 200).await?;
        let v: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("LLM returned non-JSON: {raw}"))?;
        Ok((
            v["score"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0),
            v["feedback"].as_str().unwrap_or("Review the edge cases.").to_string(),
        ))
    }

    pub async fn run_hint(&self, prompt: &str) -> anyhow::Result<String> {
        self.chat(vec![
            ChatMessage::system("You are a patient Socratic mentor. Give a short hint only."),
            ChatMessage::user(prompt),
        ], 80).await
    }
}

// ── Chain builder ─────────────────────────────────────────────────────────────

fn build_chain() -> Vec<ActiveProvider> {
    let mut chain = Vec::new();

    macro_rules! add {
        ($kind:expr, $base:expr, $models:expr) => {
            if let Some(key) = $kind.env_var().and_then(|v| std::env::var(v).ok()) {
                if let Some(m) = $models.first() {
                    chain.push(ActiveProvider { kind: $kind, api_key: key,
                        base_url: $base.to_string(), model_id: m.id });
                }
            }
        };
    }

    add!(ProviderKind::Groq,       groq::BASE_URL,       GROQ_MODELS);
    add!(ProviderKind::SambaNova,  sambanova::BASE_URL,   SAMBANOVA_MODELS);
    add!(ProviderKind::Llm7,       llm7::BASE_URL,        LLM7_MODELS);
    add!(ProviderKind::OpenRouter, openrouter::BASE_URL,  OPENROUTER_MODELS);
    add!(ProviderKind::Nvidia,     nvidia::BASE_URL,      NVIDIA_MODELS);

    // Ollama is always the last-resort local fallback.
    if let Some(m) = OLLAMA_MODELS.first() {
        chain.push(ActiveProvider { kind: ProviderKind::Ollama, api_key: String::new(),
            base_url: ollama::base_url(), model_id: m.id });
    }
    chain
}
