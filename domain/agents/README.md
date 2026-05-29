# `domain/agents`

Pure `AgentStrategy` implementations — no I/O, no HTTP.

**Layer rule:** Domain code. Prompt templates only. The runtime calls the LLM.

## `AgentStrategy` trait (from `foundation/types`)

```rust
pub trait AgentStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn adapt_difficulty(&self, recent_scores: &[f64]) -> i32;
    fn needs_mentor(&self, recent_scores: &[f64]) -> bool;
    fn build_quest_prompt(&self, goal: &str, biome: &str, difficulty: i32) -> String;
    fn build_evaluation_prompt(&self, code: &str, quest_title: &str, language: &str) -> String;
    fn build_hint_prompt(&self, concept: &str, student_level: i32) -> String;
}
```

## `Orchestrator`

Selects the appropriate `AgentStrategy` for a student based on their
performance history. Returns prompt strings — never calls the LLM directly.

```rust
let orchestrator = Orchestrator::new();
let prompt = orchestrator.build_quest_prompt(&student, &biome);
// Runtime calls: state.llm.as_ref()?.run_quest_generation(&prompt).await
```

## LLM wiring

The concrete LLM call lives in `runtime/drivers/llm/FreeClient`.
`domain/agents` never imports `runtime/drivers`.
