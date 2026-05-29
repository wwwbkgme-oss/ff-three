//! LLM7.io provider constants.
//!
//! LLM7 is a free OpenAI-compatible gateway that routes requests across
//! multiple upstream providers.  Uses virtual selectors instead of real model
//! IDs ("default", "fast").
//!
//! Free token:  https://token.llm7.io/
//! Free tier:   100 req/hr · 20 req/min · no credit card
//! Set env var: LLM7_API_KEY=...

/// LLM7.io chat completions base URL (OpenAI-compatible).
pub const BASE_URL: &str = "https://api.llm7.io/v1";

/// Env var that activates this provider.
pub const ENV_VAR: &str = "LLM7_API_KEY";
