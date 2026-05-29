//! Provider sub-modules and the shared OpenAI-compatible HTTP helper.
//!
//! All six providers use the same wire format:
//!
//!   POST {base_url}/chat/completions
//!   Authorization: Bearer {api_key}
//!   Content-Type: application/json
//!
//! Deviations:
//! - OpenRouter: adds `HTTP-Referer` + `X-Title` attribution headers.
//! - Ollama: skips `Authorization` (no key).

pub mod groq;
pub mod llm7;
pub mod nvidia;
pub mod ollama;
pub mod openrouter;
pub mod sambanova;

use anyhow::{bail, Context};
use reqwest::{Client, StatusCode};
use tracing::instrument;

use crate::llm::types::{ChatMessage, ChatRequest, ChatResponse, ProviderKind};

/// Send one OpenAI-compatible chat completion request.
///
/// Returns the assistant's reply or an error.
/// A `rate_limited:{provider}` prefix in the error signals HTTP 429
/// so `FreeClient` can fall through silently to the next provider.
#[instrument(skip(http, api_key, messages), fields(provider = %provider, model = %model))]
pub async fn openai_call(
    http:       &Client,
    provider:   ProviderKind,
    base_url:   &str,
    api_key:    &str,
    model:      &str,
    messages:   &[ChatMessage],
    max_tokens: u32,
) -> anyhow::Result<String> {
    let body = ChatRequest { model, messages, max_tokens, temperature: 0.7 };

    let mut req = http
        .post(format!("{base_url}/chat/completions"))
        .header("Content-Type", "application/json")
        .json(&body);

    if provider != ProviderKind::Ollama && !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }
    if provider == ProviderKind::OpenRouter {
        req = req
            .header("HTTP-Referer", "https://forgefabrik.dev")
            .header("X-Title",      "ForgeFabrik Academy");
    }

    let resp = req.send().await
        .with_context(|| format!("{provider}: network error"))?;

    let status = resp.status();
    if status == StatusCode::TOO_MANY_REQUESTS {
        bail!("rate_limited:{provider}");
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        bail!("{provider}: HTTP {status}: {}", &body[..body.len().min(200)]);
    }

    let parsed: ChatResponse = resp.json().await
        .with_context(|| format!("{provider}: failed to parse response JSON"))?;

    parsed.choices.into_iter().next()
        .map(|c| c.message.content)
        .context("response contained no choices")
}

/// Returns `true` when the error is a rate-limit signal.
pub fn is_rate_limit(e: &anyhow::Error) -> bool {
    e.to_string().starts_with("rate_limited:")
}
