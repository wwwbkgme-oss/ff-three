//! `FreeClient` — free-tier LLM client with automatic provider failover.
//!
//! ## How it works
//!
//! `FreeClient::from_env()` scans environment variables to discover which
//! providers are configured and builds a priority-ordered chain:
//!
//!   Groq → SambaNova → LLM7 → OpenRouter → NVIDIA NIM → Ollama
//!
//! `chat()` tries each active provider in order:
//! - **Rate-limited (429)?** → silent fallback to the next provider.
//! - **Any other error?**   → log warning, try next provider.
//! - **All exhausted?**     → return an error.
//!
//! ## Drop-in replacement for `runtime/server/src/llm.rs`
//!
//! `FreeClient` exposes the same high-level methods as the existing
//! `LlmClient` (`run_quest_generation`, `run_evaluation`, `run_hint`) so
//! `runtime/server` can swap the import with zero other changes.

use anyhow::Context;
use tracing::{info, instrument, warn};

use crate::{
    catalog::{
        GROQ_MODELS, LLM7_MODELS, NVIDIA_MODELS, OLLAMA_MODELS,
        OPENROUTER_MODELS, SAMBANOVA_MODELS,
    },
    providers::{self, is_rate_limit},
    types::{ActiveProvider, ChatMessage, ProviderKind},
};

// ── Base-URL constants from each provider module ───────────────────────────────

use crate::providers::groq;
use crate::providers::llm7;
use crate::providers::nvidia;
use crate::providers::ollama;
use crate::providers::openrouter;
use crate::providers::sambanova;

// ── FreeClient ────────────────────────────────────────────────────────────────

/// Free-tier LLM client.
///
/// Construct with [`FreeClient::from_env`] (auto-detects providers) or
/// [`FreeClient::with_providers`] (explicit list).
pub struct FreeClient {
    http:  reqwest::Client,
    chain: Vec<ActiveProvider>,
}

impl FreeClient {
    /// Auto-detect configured providers from environment variables and build
    /// the failover chain.
    ///
    /// At least one provider must be configured, otherwise the chain will
    /// fall all the way through to the Ollama fallback.  If Ollama is not
    /// running, every call to `chat()` will fail.
    pub fn from_env() -> Self {
        let http = reqwest::Client::new();
        let chain = build_chain_from_env();

        if chain.is_empty() {
            warn!(
                "llm-free: no providers configured — add one of: \
                 GROQ_API_KEY, SAMBANOVA_API_KEY, LLM7_API_KEY, \
                 OPENROUTER_API_KEY, NVIDIA_API_KEY, or run `ollama serve`"
            );
        } else {
            let names: Vec<_> = chain.iter().map(|p| p.kind.to_string()).collect();
            info!("llm-free: provider chain = [{}]", names.join(" → "));
        }

        Self { http, chain }
    }

    /// Construct with an explicit list of active providers (useful in tests).
    pub fn with_providers(chain: Vec<ActiveProvider>) -> Self {
        Self { http: reqwest::Client::new(), chain }
    }

    /// Returns `true` when at least one provider is configured.
    pub fn has_providers(&self) -> bool { !self.chain.is_empty() }

    /// Returns the list of active providers in failover order.
    pub fn providers(&self) -> &[ActiveProvider] { &self.chain }

    // ── Core request ──────────────────────────────────────────────────────────

    /// Send a chat completion request, trying each provider in order.
    ///
    /// Silently falls through to the next provider on rate-limits.
    /// Logs a warning and continues on other errors.
    /// Returns the first successful response.
    #[instrument(skip(self, messages))]
    pub async fn chat(
        &self,
        messages:   Vec<ChatMessage>,
        max_tokens: u32,
    ) -> anyhow::Result<String> {
        if self.chain.is_empty() {
            anyhow::bail!(
                "llm-free: no providers configured — \
                 set GROQ_API_KEY, SAMBANOVA_API_KEY, LLM7_API_KEY, \
                 OPENROUTER_API_KEY, or NVIDIA_API_KEY"
            );
        }

        let mut last_err = String::new();

        for provider in &self.chain {
            match providers::openai_chat(
                &self.http,
                provider.kind,
                &provider.base_url,
                &provider.api_key,
                provider.model_id,
                &messages,
                max_tokens,
            )
            .await
            {
                Ok(text) => return Ok(text),

                Err(e) if is_rate_limit(&e) => {
                    warn!(
                        provider = %provider.kind,
                        "rate-limited — trying next provider"
                    );
                    last_err = e.to_string();
                }

                Err(e) => {
                    warn!(
                        provider = %provider.kind,
                        error    = %e,
                        "provider error — trying next"
                    );
                    last_err = e.to_string();
                }
            }
        }

        anyhow::bail!(
            "all free LLM providers failed (last error: {last_err})"
        )
    }

    // ── Domain-specific helpers (drop-in for runtime/server LlmClient) ────────

    /// Generate a quest `(title, description)` from a prompt.
    pub async fn run_quest_generation(
        &self,
        prompt: &str,
    ) -> anyhow::Result<(String, String)> {
        let raw = self.chat(vec![
            ChatMessage::system(
                "You are a game designer for an educational platform. \
                 Respond with valid JSON: {\"title\": \"...\", \"description\": \"...\"}",
            ),
            ChatMessage::user(prompt),
        ], 300).await?;

        let v: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("LLM returned non-JSON: {raw}"))?;

        Ok((
            v["title"]      .as_str().unwrap_or("Learning Quest").to_string(),
            v["description"].as_str().unwrap_or("Complete this quest.").to_string(),
        ))
    }

    /// Evaluate a code submission; returns `(score 0–1, feedback)`.
    pub async fn run_evaluation(
        &self,
        prompt: &str,
    ) -> anyhow::Result<(f64, String)> {
        let raw = self.chat(vec![
            ChatMessage::system(
                "You are a code review expert. \
                 Respond with valid JSON: {\"score\": 0.0-1.0, \"feedback\": \"...\"}",
            ),
            ChatMessage::user(prompt),
        ], 200).await?;

        let v: serde_json::Value = serde_json::from_str(&raw)
            .with_context(|| format!("LLM returned non-JSON: {raw}"))?;

        Ok((
            v["score"]   .as_f64().unwrap_or(0.5).clamp(0.0, 1.0),
            v["feedback"].as_str().unwrap_or("Review the edge cases.").to_string(),
        ))
    }

    /// Generate a Socratic hint for a concept.
    pub async fn run_hint(&self, prompt: &str) -> anyhow::Result<String> {
        self.chat(vec![
            ChatMessage::system("You are a patient Socratic mentor. Give a short hint only."),
            ChatMessage::user(prompt),
        ], 80).await
    }
}

// ── Chain builder ─────────────────────────────────────────────────────────────

/// Build the ordered provider chain from environment variables.
///
/// Priority order (fastest/most-reliable first):
///   Groq → SambaNova → LLM7 → OpenRouter → NVIDIA NIM → Ollama
fn build_chain_from_env() -> Vec<ActiveProvider> {
    let mut chain: Vec<ActiveProvider> = Vec::new();

    // 1. Groq
    if let Ok(key) = std::env::var(groq::ENV_VAR) {
        if let Some(model) = GROQ_MODELS.first() {
            chain.push(ActiveProvider {
                kind:     ProviderKind::Groq,
                api_key:  key,
                base_url: groq::BASE_URL.to_string(),
                model_id: model.id,
            });
        }
    }

    // 2. SambaNova
    if let Ok(key) = std::env::var(sambanova::ENV_VAR) {
        if let Some(model) = SAMBANOVA_MODELS.first() {
            chain.push(ActiveProvider {
                kind:     ProviderKind::SambaNova,
                api_key:  key,
                base_url: sambanova::BASE_URL.to_string(),
                model_id: model.id,
            });
        }
    }

    // 3. LLM7
    if let Ok(key) = std::env::var(llm7::ENV_VAR) {
        if let Some(model) = LLM7_MODELS.first() {
            chain.push(ActiveProvider {
                kind:     ProviderKind::Llm7,
                api_key:  key,
                base_url: llm7::BASE_URL.to_string(),
                model_id: model.id,
            });
        }
    }

    // 4. OpenRouter
    if let Ok(key) = std::env::var(openrouter::ENV_VAR) {
        if let Some(model) = OPENROUTER_MODELS.first() {
            chain.push(ActiveProvider {
                kind:     ProviderKind::OpenRouter,
                api_key:  key,
                base_url: openrouter::BASE_URL.to_string(),
                model_id: model.id,
            });
        }
    }

    // 5. NVIDIA NIM
    if let Ok(key) = std::env::var(nvidia::ENV_VAR) {
        if let Some(model) = NVIDIA_MODELS.first() {
            chain.push(ActiveProvider {
                kind:     ProviderKind::Nvidia,
                api_key:  key,
                base_url: nvidia::BASE_URL.to_string(),
                model_id: model.id,
            });
        }
    }

    // 6. Ollama — always appended as last-resort fallback.
    //    `openai_chat` will return a network error if `ollama serve` is not
    //    running; the caller treats that as "try next" (there is no next).
    if let Some(model) = OLLAMA_MODELS.first() {
        chain.push(ActiveProvider {
            kind:     ProviderKind::Ollama,
            api_key:  String::new(),
            base_url: ollama::base_url(),
            model_id: model.id,
        });
    }

    chain
}
