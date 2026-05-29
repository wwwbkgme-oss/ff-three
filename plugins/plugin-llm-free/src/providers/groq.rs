//! Groq provider constants.
//!
//! Groq Cloud runs open-weight models on custom LPU hardware, delivering
//! sub-100 ms TTFT.  The free tier is very generous: 14 400 req/day.
//!
//! Sign up (free): https://console.groq.com/keys
//! Set env var:    GROQ_API_KEY=gsk_...

/// Groq chat completions base URL (OpenAI-compatible).
pub const BASE_URL: &str = "https://api.groq.com/openai/v1";

/// Env var that activates this provider.
pub const ENV_VAR: &str = "GROQ_API_KEY";
