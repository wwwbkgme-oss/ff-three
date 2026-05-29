//! OpenRouter provider constants.
//! Sign up (free): https://openrouter.ai/keys
//! Set: OPENROUTER_API_KEY=sk-or-v1-...
//!
//! Requires extra headers (added by openai_call):
//!   HTTP-Referer: https://forgefabrik.dev
//!   X-Title: ForgeFabrik Academy
pub const BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const ENV_VAR:  &str = "OPENROUTER_API_KEY";
