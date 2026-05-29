//! Shared types for the free-provider layer.

use serde::{Deserialize, Serialize};

// ── ChatMessage ───────────────────────────────────────────────────────────────

/// An OpenAI-style chat message (role + content).
///
/// All six providers in this crate speak the same wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(s: impl Into<String>) -> Self { Self { role: "system".into(), content: s.into() } }
    pub fn user(s: impl Into<String>)   -> Self { Self { role: "user".into(),   content: s.into() } }
}

// ── ProviderKind ──────────────────────────────────────────────────────────────

/// Identifier for one free-tier provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    /// Groq Cloud — fastest free inference (<100 ms TTFT).
    /// Sign up: https://console.groq.com/keys
    Groq,
    /// SambaNova Cloud — fast RDU inference, no credit card.
    /// Sign up: https://cloud.sambanova.ai/
    SambaNova,
    /// LLM7.io — free gateway, routes across providers.
    /// Token: https://token.llm7.io/
    Llm7,
    /// OpenRouter — `:free` model suffix = $0.00 / token.
    /// Sign up: https://openrouter.ai/keys
    OpenRouter,
    /// NVIDIA NIM — 1 000 free requests/month.
    /// Sign up: https://build.nvidia.com
    Nvidia,
    /// Ollama local — always free, requires `ollama serve` running.
    Ollama,
}

impl ProviderKind {
    pub fn env_var(&self) -> Option<&'static str> {
        match self {
            ProviderKind::Groq        => Some("GROQ_API_KEY"),
            ProviderKind::SambaNova   => Some("SAMBANOVA_API_KEY"),
            ProviderKind::Llm7        => Some("LLM7_API_KEY"),
            ProviderKind::OpenRouter  => Some("OPENROUTER_API_KEY"),
            ProviderKind::Nvidia      => Some("NVIDIA_API_KEY"),
            ProviderKind::Ollama      => None, // no key needed
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ProviderKind::Groq       => "Groq",
            ProviderKind::SambaNova  => "SambaNova",
            ProviderKind::Llm7       => "LLM7",
            ProviderKind::OpenRouter => "OpenRouter",
            ProviderKind::Nvidia     => "NVIDIA NIM",
            ProviderKind::Ollama     => "Ollama (local)",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

// ── Model ──────────────────────────────────────────────────────────────────────

/// A known-free model on one provider.
#[derive(Debug, Clone)]
pub struct Model {
    pub provider:    ProviderKind,
    /// Model ID as expected by the provider's API.
    pub id:          &'static str,
    /// Human-readable display name.
    pub name:        &'static str,
    /// Maximum context window in tokens.
    pub context_len: u32,
    /// One-line note about the free tier limits.
    pub free_notes:  &'static str,
}

// ── ProviderEntry (runtime config) ───────────────────────────────────────────

/// A provider that is active at runtime (key present + model selected).
#[derive(Debug, Clone)]
pub struct ActiveProvider {
    pub kind:     ProviderKind,
    /// API key (empty string for Ollama).
    pub api_key:  String,
    /// Base URL for chat completions (without trailing `/chat/completions`).
    pub base_url: String,
    /// The model ID to use (from the catalog).
    pub model_id: &'static str,
}

// ── Wire types (OpenAI-compatible request / response) ─────────────────────────

#[derive(Serialize)]
pub(crate) struct ChatRequest<'a> {
    pub model:       &'a str,
    pub messages:    &'a [ChatMessage],
    pub max_tokens:  u32,
    pub temperature: f32,
}

#[derive(Deserialize)]
pub(crate) struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
pub(crate) struct ChatChoice {
    pub message: ChatChoiceMsg,
}

#[derive(Deserialize)]
pub(crate) struct ChatChoiceMsg {
    pub content: String,
}
