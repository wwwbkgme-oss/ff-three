//! SambaNova Cloud provider constants.
//!
//! SambaNova runs inference on custom RDU chips.  The free tier requires no
//! credit card: 20-480 RPM depending on model size.
//!
//! Sign up (free): https://cloud.sambanova.ai/
//! API keys:       https://cloud.sambanova.ai/apis
//! Set env var:    SAMBANOVA_API_KEY=...

/// SambaNova chat completions base URL (OpenAI-compatible).
pub const BASE_URL: &str = "https://api.sambanova.ai/v1";

/// Env var that activates this provider.
pub const ENV_VAR: &str = "SAMBANOVA_API_KEY";
