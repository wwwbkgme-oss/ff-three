//! NVIDIA NIM provider constants.
//!
//! NVIDIA NIM hosts large open-weight models via integrate.api.nvidia.com.
//! New accounts receive 1 000 free credits/month.  Zero-cost models (flagged
//! in the catalog) do NOT consume credits.
//!
//! Sign up (free): https://build.nvidia.com
//! Set env var:    NVIDIA_API_KEY=nvapi-...

/// NVIDIA NIM chat completions base URL (OpenAI-compatible).
pub const BASE_URL: &str = "https://integrate.api.nvidia.com/v1";

/// Env var that activates this provider.
pub const ENV_VAR: &str = "NVIDIA_API_KEY";
