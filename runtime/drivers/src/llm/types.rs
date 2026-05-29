//! Shared types for the LLM driver layer.
//!
//! These types are **infrastructure** — they describe transport concerns
//! (API keys, base URLs, model IDs, wire formats) and must never be
//! imported by `domain/` or `foundation/`.

use serde::{Deserialize, Serialize};

// ── ChatMessage (OpenAI wire format) ──────────────────────────────────────────

/// An OpenAI-compatible chat message (role + content).
/// All six providers in this driver share the same wire format.
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

/// Infrastructure identity of one free-tier LLM provider.
///
/// ## Architectural note
///
/// `ProviderKind` is a **driver-layer type** — it describes transport
/// infrastructure.  Domain code must never import it.  If the domain needs
/// to express "use an LLM", it does so through the abstract `AgentStrategy`
/// trait from `foundation/types`, and the runtime wires the concrete driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    /// Groq Cloud — <100 ms TTFT, 14 400 req/day free.
    Groq,
    /// SambaNova Cloud — fast RDU inference, no credit card required.
    SambaNova,
    /// LLM7.io — free OpenAI-compatible gateway, 100 req/hr.
    Llm7,
    /// OpenRouter — `:free` model suffix = $0.00/token.
    OpenRouter,
    /// NVIDIA NIM — 1 000 free requests/month.
    Nvidia,
    /// Ollama local — always free, requires `ollama serve`.
    Ollama,
}

impl ProviderKind {
    pub fn env_var(&self) -> Option<&'static str> {
        match self {
            Self::Groq       => Some("GROQ_API_KEY"),
            Self::SambaNova  => Some("SAMBANOVA_API_KEY"),
            Self::Llm7       => Some("LLM7_API_KEY"),
            Self::OpenRouter => Some("OPENROUTER_API_KEY"),
            Self::Nvidia     => Some("NVIDIA_API_KEY"),
            Self::Ollama     => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Groq       => "Groq",
            Self::SambaNova  => "SambaNova",
            Self::Llm7       => "LLM7",
            Self::OpenRouter => "OpenRouter",
            Self::Nvidia     => "NVIDIA NIM",
            Self::Ollama     => "Ollama (local)",
        }
    }
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

// ── ActiveProvider (runtime config) ──────────────────────────────────────────

/// A configured provider ready to receive requests.
#[derive(Debug, Clone)]
pub struct ActiveProvider {
    pub kind:     ProviderKind,
    /// API key (empty for Ollama).
    pub api_key:  String,
    /// Base URL for the chat completions endpoint.
    pub base_url: String,
    /// Model ID to use (from the catalog).
    pub model_id: &'static str,
}

// ── OpenAI-compatible wire types ──────────────────────────────────────────────

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
