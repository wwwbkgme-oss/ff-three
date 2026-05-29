# `runtime/drivers`

I/O adapter layer — infrastructure implementations that bridge domain traits
with the outside world.

**Layer rule:** I/O is allowed here. Never imported by `domain/` or `plugins/`.

## Current adapters

### `llm/` — Free-tier LLM provider chain

`FreeClient` implements the same interface as the old `LlmClient` (chat,
run_quest_generation, run_evaluation, run_hint) but routes through 6
free providers in priority order:

| # | Provider | Free tier | Env var |
|---|---|---|---|
| 1 | **Groq** | 14 400 req/day, <100ms TTFT | `GROQ_API_KEY` |
| 2 | **SambaNova** | 20–480 RPM, no CC | `SAMBANOVA_API_KEY` |
| 3 | **LLM7** | 100 req/hr | `LLM7_API_KEY` |
| 4 | **OpenRouter** | `:free` models, $0/token | `OPENROUTER_API_KEY` |
| 5 | **NVIDIA NIM** | 1 000 req/month | `NVIDIA_API_KEY` |
| 6 | **Ollama** | local, always free | `OLLAMA_HOST` |

`FreeClient::from_env()` auto-detects configured providers.
`chat()` falls back silently on 429s.

## Structure

```
src/
├── lib.rs         — pub use llm::FreeClient
└── llm/
    ├── mod.rs     — FreeClient + build_chain()
    ├── types.rs   — ChatMessage, ProviderKind, ActiveProvider
    ├── catalog.rs — verified-free model lists
    └── providers/
        ├── mod.rs          — openai_call() shared HTTP helper
        ├── groq.rs
        ├── sambanova.rs
        ├── llm7.rs
        ├── openrouter.rs
        ├── nvidia.rs
        └── ollama.rs
```

## Adding a new provider

1. Add `src/llm/providers/<name>.rs` with `BASE_URL` + `ENV_VAR` constants
2. Add model list to `src/llm/catalog.rs`
3. Extend `build_chain()` in `src/llm/mod.rs`
4. Update this README's table

**Criterion:** permanently free, no credit card, no trial expiry.

## Boundary spec

See [`docs/PLUGIN_VS_DRIVER.md`](../../docs/PLUGIN_VS_DRIVER.md).
