//! OpenRouter provider constants.
//!
//! OpenRouter aggregates models from many providers.  Models with the `:free`
//! suffix are permanently $0.00/token — no credit-card ever.
//!
//! Sign up (free): https://openrouter.ai/keys
//! Set env var:    OPENROUTER_API_KEY=sk-or-v1-...
//!
//! Required extra headers (added automatically by `openai_chat`):
//!   HTTP-Referer: https://forgefabrik.dev
//!   X-Title:      ForgeFabrik Academy

/// OpenRouter chat completions base URL.
pub const BASE_URL: &str = "https://openrouter.ai/api/v1";

/// Env var that activates this provider.
pub const ENV_VAR: &str = "OPENROUTER_API_KEY";
