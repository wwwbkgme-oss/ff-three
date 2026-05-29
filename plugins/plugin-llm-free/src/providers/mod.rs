//! Provider modules and the shared OpenAI-compatible HTTP helper.
//!
//! All six providers in this crate speak the same wire format:
//!
//!   POST {base_url}/chat/completions
//!   Authorization: Bearer {api_key}
//!   Content-Type: application/json
//!   { "model": "...", "messages": [...], "max_tokens": N, "temperature": 0.7 }
//!
//! The only deviations are:
//! - **OpenRouter** requires two extra headers (`HTTP-Referer`, `X-Title`).
//! - **Ollama** skips the `Authorization` header (no key).
//! - **NVIDIA NIM** uses `nvapi-` prefixed keys but otherwise standard bearer.

pub mod groq;
pub mod llm7;
pub mod nvidia;
pub mod ollama;
pub mod openrouter;
pub mod sambanova;

use anyhow::{Context, bail};
use reqwest::{Client, StatusCode};
use tracing::instrument;

use crate::types::{ChatMessage, ChatRequest, ChatResponse, ProviderKind};

/// Send one OpenAI-compatible chat-completion request.
///
/// Returns the assistant's reply text on success, or a descriptive error.
/// The caller (`FreeClient`) inspects the error to decide whether to retry
/// on the next provider.
#[instrument(skip(http, api_key, messages), fields(provider = %provider, model = %model))]
pub async fn openai_chat(
    http:      &Client,
    provider:  ProviderKind,
    base_url:  &str,
    api_key:   &str,
    model:     &str,
    messages:  &[ChatMessage],
    max_tokens: u32,
) -> anyhow::Result<String> {
    let body = ChatRequest {
        model,
        messages,
        max_tokens,
        temperature: 0.7,
    };

    let mut req = http
        .post(format!("{base_url}/chat/completions"))
        .header("Content-Type", "application/json")
        .json(&body);

    // Auth: skip for Ollama (no key), use Bearer for all others.
    if provider != ProviderKind::Ollama && !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    // OpenRouter requires attribution headers.
    if provider == ProviderKind::OpenRouter {
        req = req
            .header("HTTP-Referer", "https://forgefabrik.dev")
            .header("X-Title",      "ForgeFabrik Academy");
    }

    let resp = req.send().await
        .with_context(|| format!("{provider}: network error"))?;

    let status = resp.status();

    // Surface rate-limit and server errors as typed errors so the caller can
    // decide whether to fall through to the next provider.
    if status == StatusCode::TOO_MANY_REQUESTS {
        bail!("rate_limited:{provider}");
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("{provider}: HTTP {status}: {}", truncate(&body, 200));
    }

    let parsed: ChatResponse = resp
        .json()
        .await
        .with_context(|| format!("{provider}: failed to parse response JSON"))?;

    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .context("response contained no choices")
}

/// Returns `true` when the error string signals a rate-limit.
///
/// `FreeClient` uses this to skip silently to the next provider instead of
/// surfacing the error to the caller.
pub fn is_rate_limit(err: &anyhow::Error) -> bool {
    err.to_string().starts_with("rate_limited:")
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max { s } else { &s[..max] }
}
