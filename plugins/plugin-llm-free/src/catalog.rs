//! Hard-coded catalog of known-free models, sourced from pi-free and verified
//! against each provider's documentation (May 2026).
//!
//! **Maintenance:** When a provider drops a model from their free tier,
//! remove it here.  The `FreeClient` picks the first model in each provider's
//! list as the default.

use crate::types::{Model, ProviderKind};

// ── Groq ──────────────────────────────────────────────────────────────────────
// Free tier: 14 400 req/day · 6 000 tokens/min · no credit card
// https://console.groq.com/docs/rate-limits

pub const GROQ_MODELS: &[Model] = &[
    Model {
        provider:    ProviderKind::Groq,
        id:          "llama-3.1-8b-instant",
        name:        "Llama 3.1 8B Instant",
        context_len: 131_072,
        free_notes:  "6 000 tok/min, 14 400 req/day",
    },
    Model {
        provider:    ProviderKind::Groq,
        id:          "gemma2-9b-it",
        name:        "Gemma 2 9B IT",
        context_len: 8_192,
        free_notes:  "14 400 req/day",
    },
    Model {
        provider:    ProviderKind::Groq,
        id:          "llama3-8b-8192",
        name:        "Llama 3 8B",
        context_len: 8_192,
        free_notes:  "30 000 tok/min, 14 400 req/day",
    },
    Model {
        provider:    ProviderKind::Groq,
        id:          "mixtral-8x7b-32768",
        name:        "Mixtral 8x7B",
        context_len: 32_768,
        free_notes:  "5 000 tok/min, 14 400 req/day",
    },
];

// ── SambaNova ─────────────────────────────────────────────────────────────────
// Free tier: 20-480 RPM · no credit card required · forever free
// https://community.sambanova.ai/t/rate-limits

pub const SAMBANOVA_MODELS: &[Model] = &[
    Model {
        provider:    ProviderKind::SambaNova,
        id:          "Meta-Llama-3.3-70B-Instruct",
        name:        "Llama 3.3 70B Instruct",
        context_len: 131_072,
        free_notes:  "20 RPM / 400 RPD free",
    },
    Model {
        provider:    ProviderKind::SambaNova,
        id:          "Meta-Llama-3.1-8B-Instruct",
        name:        "Llama 3.1 8B Instruct",
        context_len: 16_384,
        free_notes:  "480 RPM / 9 600 RPD free",
    },
    Model {
        provider:    ProviderKind::SambaNova,
        id:          "Qwen3-32B",
        name:        "Qwen 3 32B",
        context_len: 131_072,
        free_notes:  "Free tier (verify at sambanova.ai)",
    },
    Model {
        provider:    ProviderKind::SambaNova,
        id:          "DeepSeek-V3-0324",
        name:        "DeepSeek V3",
        context_len: 65_536,
        free_notes:  "Free tier (verify at sambanova.ai)",
    },
];

// ── LLM7.io ───────────────────────────────────────────────────────────────────
// Free tier: 100 req/hr · 20 req/min · free token at token.llm7.io
// Uses virtual selectors, not real model IDs.

pub const LLM7_MODELS: &[Model] = &[
    Model {
        provider:    ProviderKind::Llm7,
        id:          "default",
        name:        "LLM7 Default (first available free)",
        context_len: 32_000,
        free_notes:  "100 req/hr · 20 req/min",
    },
    Model {
        provider:    ProviderKind::Llm7,
        id:          "fast",
        name:        "LLM7 Fast (lowest latency)",
        context_len: 32_000,
        free_notes:  "100 req/hr · 20 req/min",
    },
];

// ── OpenRouter ────────────────────────────────────────────────────────────────
// Models with `:free` suffix are permanently $0.00/token.
// Free account required: https://openrouter.ai/keys

pub const OPENROUTER_MODELS: &[Model] = &[
    Model {
        provider:    ProviderKind::OpenRouter,
        id:          "deepseek/deepseek-chat-v3-0324:free",
        name:        "DeepSeek V3 (free)",
        context_len: 163_840,
        free_notes:  "$0 / token, free OpenRouter account",
    },
    Model {
        provider:    ProviderKind::OpenRouter,
        id:          "deepseek/deepseek-r1:free",
        name:        "DeepSeek R1 (free)",
        context_len: 163_840,
        free_notes:  "$0 / token, free OpenRouter account",
    },
    Model {
        provider:    ProviderKind::OpenRouter,
        id:          "meta-llama/llama-3.1-8b-instruct:free",
        name:        "Llama 3.1 8B (free)",
        context_len: 131_072,
        free_notes:  "$0 / token",
    },
    Model {
        provider:    ProviderKind::OpenRouter,
        id:          "google/gemma-3-12b-it:free",
        name:        "Gemma 3 12B (free)",
        context_len: 131_072,
        free_notes:  "$0 / token",
    },
    Model {
        provider:    ProviderKind::OpenRouter,
        id:          "mistralai/mistral-7b-instruct:free",
        name:        "Mistral 7B Instruct (free)",
        context_len: 32_768,
        free_notes:  "$0 / token",
    },
    Model {
        provider:    ProviderKind::OpenRouter,
        id:          "qwen/qwen3-8b:free",
        name:        "Qwen3 8B (free)",
        context_len: 40_960,
        free_notes:  "$0 / token",
    },
];

// ── NVIDIA NIM ────────────────────────────────────────────────────────────────
// 1 000 free credits/month.  Zero-cost models don't consume credits.
// https://build.nvidia.com

pub const NVIDIA_MODELS: &[Model] = &[
    Model {
        provider:    ProviderKind::Nvidia,
        id:          "meta/llama-3.1-8b-instruct",
        name:        "Llama 3.1 8B Instruct (NVIDIA NIM)",
        context_len: 131_072,
        free_notes:  "Zero-cost model, 1 000 credits/month",
    },
    Model {
        provider:    ProviderKind::Nvidia,
        id:          "meta/llama-3.2-1b-instruct",
        name:        "Llama 3.2 1B Instruct (NVIDIA NIM)",
        context_len: 131_072,
        free_notes:  "Zero-cost model",
    },
    Model {
        provider:    ProviderKind::Nvidia,
        id:          "mistralai/mistral-7b-instruct-v0.3",
        name:        "Mistral 7B Instruct v0.3 (NVIDIA NIM)",
        context_len: 32_768,
        free_notes:  "Zero-cost model",
    },
];

// ── Ollama (local) ────────────────────────────────────────────────────────────
// Completely free — requires `ollama serve` running locally.
// Models vary by what the user has pulled; we default to common ones.

pub const OLLAMA_MODELS: &[Model] = &[
    Model {
        provider:    ProviderKind::Ollama,
        id:          "llama3.2",
        name:        "Llama 3.2 (local)",
        context_len: 131_072,
        free_notes:  "Local inference, pull with: ollama pull llama3.2",
    },
    Model {
        provider:    ProviderKind::Ollama,
        id:          "llama3.1",
        name:        "Llama 3.1 (local)",
        context_len: 131_072,
        free_notes:  "Local inference, pull with: ollama pull llama3.1",
    },
    Model {
        provider:    ProviderKind::Ollama,
        id:          "gemma3",
        name:        "Gemma 3 (local)",
        context_len: 131_072,
        free_notes:  "Local inference, pull with: ollama pull gemma3",
    },
];

// ── Unified catalog ───────────────────────────────────────────────────────────

/// All known-free models from all providers, ordered by provider priority.
pub const PROVIDER_CATALOG: &[&[Model]] = &[
    GROQ_MODELS,
    SAMBANOVA_MODELS,
    LLM7_MODELS,
    OPENROUTER_MODELS,
    NVIDIA_MODELS,
    OLLAMA_MODELS,
];
