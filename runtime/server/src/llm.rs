//! OpenAI LLM client – runtime I/O for AI agent prompt execution.
//!
//! The domain `Orchestrator` builds prompt strings; this client sends them
//! to the API and parses structured responses.

use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Serialize)]
struct ChatRequest<'a> {
    model:      &'a str,
    messages:   Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role:    String,
    pub content: String,
}

impl ChatMessage {
    pub fn system(s: impl Into<String>) -> Self { Self { role: "system".into(), content: s.into() } }
    pub fn user(s: impl Into<String>)   -> Self { Self { role: "user".into(),   content: s.into() } }
}

#[derive(Deserialize)]
struct ChatResponse { choices: Vec<Choice> }
#[derive(Deserialize)]
struct Choice { message: ChoiceMsg }
#[derive(Deserialize)]
struct ChoiceMsg { content: String }

pub struct LlmClient {
    api_key: String,
    model:   String,
    http:    reqwest::Client,
}

impl LlmClient {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), model: model.into(), http: reqwest::Client::new() }
    }

    /// Low-level chat completions call.
    #[instrument(skip(self, messages), fields(model = %self.model))]
    pub async fn chat(&self, messages: Vec<ChatMessage>, max_tokens: u32) -> anyhow::Result<String> {
        let req = ChatRequest {
            model: &self.model, messages, max_tokens, temperature: 0.7,
        };
        let resp = self.http
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&req)
            .send().await?
            .error_for_status()?
            .json::<ChatResponse>().await?;

        resp.choices.into_iter().next()
            .map(|c| c.message.content)
            .ok_or_else(|| anyhow::anyhow!("OpenAI returned no choices"))
    }

    // ── Domain-specific wrappers ──────────────────────────────────────────────

    /// Run a quest-generation prompt; returns `(title, description)`.
    pub async fn run_quest_generation(&self, prompt: &str) -> anyhow::Result<(String, String)> {
        let raw = self.chat(vec![
            ChatMessage::system("You are a game designer for an educational platform."),
            ChatMessage::user(prompt),
        ], 300).await?;
        let v: serde_json::Value = serde_json::from_str(&raw)?;
        Ok((
            v["title"].as_str().unwrap_or("Learning Quest").to_string(),
            v["description"].as_str().unwrap_or("Complete this quest.").to_string(),
        ))
    }

    /// Run an evaluation prompt; returns `(score 0–1, feedback)`.
    pub async fn run_evaluation(&self, prompt: &str) -> anyhow::Result<(f64, String)> {
        let raw = self.chat(vec![
            ChatMessage::system("You are a code review expert."),
            ChatMessage::user(prompt),
        ], 200).await?;
        let v: serde_json::Value = serde_json::from_str(&raw)?;
        Ok((
            v["score"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0),
            v["feedback"].as_str().unwrap_or("Review the edge cases.").to_string(),
        ))
    }

    /// Run a hint prompt; returns the hint string.
    pub async fn run_hint(&self, prompt: &str) -> anyhow::Result<String> {
        self.chat(vec![
            ChatMessage::system("You are a patient Socratic mentor."),
            ChatMessage::user(prompt),
        ], 80).await
    }
}
