//! Hard-coded catalog of verified-free models, sourced from pi-free
//! (https://github.com/apmantza/pi-free) and cross-checked against each
//! provider's documentation (May 2026).
//!
//! **Maintenance:** When a provider drops a model from their free tier,
//! remove it here.  `FreeClient` picks the first entry per provider.

use crate::llm::types::ProviderKind;

// ── Model struct ──────────────────────────────────────────────────────────────

/// A known-free model on one provider.
#[derive(Debug, Clone)]
pub struct Model {
    pub provider:    ProviderKind,
    /// Model ID as expected by the provider API.
    pub id:          &'static str,
    pub name:        &'static str,
    pub context_len: u32,
    /// One-line description of free-tier limits.
    pub free_notes:  &'static str,
}

// ── Groq ──────────────────────────────────────────────────────────────────────
// Free: 14 400 req/day · 6 000 tok/min · no credit card
// Docs: https://console.groq.com/docs/rate-limits

pub const GROQ_MODELS: &[Model] = &[
    Model { provider: ProviderKind::Groq,
        id: "llama-3.1-8b-instant",    name: "Llama 3.1 8B Instant",
        context_len: 131_072, free_notes: "6 000 tok/min, 14 400 req/day" },
    Model { provider: ProviderKind::Groq,
        id: "gemma2-9b-it",             name: "Gemma 2 9B IT",
        context_len: 8_192,   free_notes: "14 400 req/day" },
    Model { provider: ProviderKind::Groq,
        id: "mixtral-8x7b-32768",       name: "Mixtral 8x7B",
        context_len: 32_768,  free_notes: "5 000 tok/min, 14 400 req/day" },
];

// ── SambaNova ─────────────────────────────────────────────────────────────────
// Free: 20-480 RPM · no credit card required · forever free
// Docs: https://community.sambanova.ai/t/rate-limits

pub const SAMBANOVA_MODELS: &[Model] = &[
    Model { provider: ProviderKind::SambaNova,
        id: "Meta-Llama-3.3-70B-Instruct", name: "Llama 3.3 70B Instruct",
        context_len: 131_072, free_notes: "20 RPM / 400 RPD free" },
    Model { provider: ProviderKind::SambaNova,
        id: "Meta-Llama-3.1-8B-Instruct",  name: "Llama 3.1 8B Instruct",
        context_len: 16_384,  free_notes: "480 RPM / 9 600 RPD free" },
    Model { provider: ProviderKind::SambaNova,
        id: "DeepSeek-V3-0324",             name: "DeepSeek V3",
        context_len: 65_536,  free_notes: "Free tier" },
];

// ── LLM7.io ───────────────────────────────────────────────────────────────────
// Free: 100 req/hr · 20 req/min · free token at https://token.llm7.io/

pub const LLM7_MODELS: &[Model] = &[
    Model { provider: ProviderKind::Llm7,
        id: "default", name: "LLM7 Default (first available free)",
        context_len: 32_000, free_notes: "100 req/hr · 20 req/min" },
    Model { provider: ProviderKind::Llm7,
        id: "fast",    name: "LLM7 Fast (lowest latency)",
        context_len: 32_000, free_notes: "100 req/hr · 20 req/min" },
];

// ── OpenRouter ────────────────────────────────────────────────────────────────
// `:free` suffix = $0.00/token permanently.
// Sign up: https://openrouter.ai/keys

pub const OPENROUTER_MODELS: &[Model] = &[
    Model { provider: ProviderKind::OpenRouter,
        id: "deepseek/deepseek-chat-v3-0324:free", name: "DeepSeek V3 (free)",
        context_len: 163_840, free_notes: "$0/token, free account" },
    Model { provider: ProviderKind::OpenRouter,
        id: "deepseek/deepseek-r1:free",           name: "DeepSeek R1 (free)",
        context_len: 163_840, free_notes: "$0/token" },
    Model { provider: ProviderKind::OpenRouter,
        id: "meta-llama/llama-3.1-8b-instruct:free", name: "Llama 3.1 8B (free)",
        context_len: 131_072, free_notes: "$0/token" },
    Model { provider: ProviderKind::OpenRouter,
        id: "google/gemma-3-12b-it:free",          name: "Gemma 3 12B (free)",
        context_len: 131_072, free_notes: "$0/token" },
    Model { provider: ProviderKind::OpenRouter,
        id: "qwen/qwen3-8b:free",                  name: "Qwen3 8B (free)",
        context_len: 40_960,  free_notes: "$0/token" },
];

// ── NVIDIA NIM ────────────────────────────────────────────────────────────────
// 1 000 free credits/month.  Zero-cost models don't consume credits.
// Sign up: https://build.nvidia.com

pub const NVIDIA_MODELS: &[Model] = &[
    Model { provider: ProviderKind::Nvidia,
        id: "meta/llama-3.1-8b-instruct",        name: "Llama 3.1 8B (NVIDIA NIM)",
        context_len: 131_072, free_notes: "Zero-cost model" },
    Model { provider: ProviderKind::Nvidia,
        id: "mistralai/mistral-7b-instruct-v0.3", name: "Mistral 7B v0.3 (NVIDIA NIM)",
        context_len: 32_768,  free_notes: "Zero-cost model" },
];

// ── Ollama (local) ────────────────────────────────────────────────────────────

pub const OLLAMA_MODELS: &[Model] = &[
    Model { provider: ProviderKind::Ollama,
        id: "llama3.2", name: "Llama 3.2 (local)",
        context_len: 131_072, free_notes: "pull: ollama pull llama3.2" },
    Model { provider: ProviderKind::Ollama,
        id: "gemma3",   name: "Gemma 3 (local)",
        context_len: 131_072, free_notes: "pull: ollama pull gemma3" },
];

// ── Unified slice (priority order) ───────────────────────────────────────────

pub const ALL_PROVIDERS: &[&[Model]] = &[
    GROQ_MODELS,
    SAMBANOVA_MODELS,
    LLM7_MODELS,
    OPENROUTER_MODELS,
    NVIDIA_MODELS,
    OLLAMA_MODELS,
];
