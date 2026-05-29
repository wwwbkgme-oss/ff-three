//! Ollama local provider constants.
//! Install: https://ollama.com/ — then `ollama serve`
//! Override host: OLLAMA_HOST=http://myserver:11434
pub fn base_url() -> String {
    std::env::var("OLLAMA_HOST")
        .unwrap_or_else(|_| "http://localhost:11434".into())
        + "/v1"
}
