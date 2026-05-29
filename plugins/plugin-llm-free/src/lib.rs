//! `llm-free` — free-tier LLM provider chain for ForgeFabrik Academy.
//!
//! Provides a `FreeClient` that tries multiple **zero-cost** LLM providers
//! in priority order, falling back automatically on rate-limits and errors.
//!
//! ## Provider priority (highest → lowest)
//!
//! | # | Provider   | Free tier                          | Requires                     |
//! |---|------------|------------------------------------|------------------------------|
//! | 1 | Groq       | 14 400 req/day, 6 000 tok/min      | `GROQ_API_KEY` (free account)|
//! | 2 | SambaNova  | 20-480 RPM, no credit card         | `SAMBANOVA_API_KEY`          |
//! | 3 | LLM7       | 100 req/hr, no credit card         | `LLM7_API_KEY`               |
//! | 4 | OpenRouter | `:free` models, 0 cost             | `OPENROUTER_API_KEY`         |
//! | 5 | NVIDIA NIM | 1 000 req/month credits            | `NVIDIA_API_KEY`             |
//! | 6 | Ollama     | Local inference, always free       | Running `ollama serve`       |
//!
//! ## Usage
//!
//! ```rust,no_run
//! use llm_free::FreeClient;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Auto-detects configured providers from env vars.
//!     let client = FreeClient::from_env();
//!     let reply = client.chat(vec![
//!         llm_free::ChatMessage::user("Explain Rust ownership in one sentence."),
//!     ], 80).await.unwrap();
//!     println!("{reply}");
//! }
//! ```
//!
//! ## Integration with runtime/server
//!
//! `FreeClient` is a drop-in replacement for `runtime/server/src/llm.rs`:
//! same `chat`, `run_quest_generation`, `run_evaluation`, `run_hint` methods.
//! Set any of the provider env vars; the rest is automatic.

pub mod catalog;
pub mod client;
pub mod providers;
pub mod types;

pub use catalog::PROVIDER_CATALOG;
pub use client::FreeClient;
pub use types::{ChatMessage, ProviderKind};
