//! Ollama local provider constants.
//!
//! Ollama runs open-weight models locally — completely free, no API key, no
//! internet required.  Requires `ollama serve` to be running on the host.
//!
//! Install: https://ollama.com/
//! Pull a model: ollama pull llama3.2
//!
//! No env var needed.  The provider is always included as the last fallback;
//! `FreeClient` will skip it gracefully if `ollama serve` is not running.

/// Ollama local chat completions base URL.
///
/// Override with `OLLAMA_HOST` env var for non-standard ports (e.g. a remote
/// Ollama server: `http://myserver:11434`).
pub fn base_url() -> String {
    std::env::var("OLLAMA_HOST")
        .unwrap_or_else(|_| "http://localhost:11434".into())
        + "/v1"
}
