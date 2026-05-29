//! Foundation trait contracts.
//!
//! These traits define the INTERFACES that domain and runtime crates implement.
//! They contain no I/O — all implementations must stay within their allowed layer.

use async_trait::async_trait;

// ── Event-sourcing contracts ──────────────────────────────────────────────────

/// Apply a single event to a state, producing the next state.
/// Implementations MUST be deterministic and side-effect free.
pub trait Reducer<S, E> {
    fn apply(state: S, event: &E) -> S;
}

/// Handle a command by validating it against the current state and emitting events.
/// Implementations MUST be deterministic and side-effect free.
pub trait CommandHandler<S, C, E> {
    type Error;
    fn handle(&self, state: &S, command: C) -> Result<Vec<E>, Self::Error>;
}

// ── Agent strategy contract ───────────────────────────────────────────────────

/// Pure, synchronous strategy contract for all Academy agents.
///
/// Methods return prompts or computed values – they do NOT perform I/O.
/// The runtime layer supplies those prompts to LLM endpoints.
#[async_trait]
pub trait AgentStrategy: Send + Sync {
    /// Human-readable agent name for logs and student-facing messages.
    fn name(&self) -> &str;

    /// Compute the next ideal difficulty (1–10) from recent scores.
    /// Deterministic, no randomness.
    fn adapt_difficulty(&self, recent_scores: &[f64]) -> i32;

    /// `true` when a student needs a mentor based on rolling performance.
    fn needs_mentor(&self, recent_scores: &[f64]) -> bool;

    /// Build a prompt asking the LLM to generate a quest.
    /// Returns a ready-to-send prompt string; does not call the LLM.
    fn build_quest_prompt(&self, goal: &str, biome: &str, difficulty: i32) -> String;

    /// Build a prompt asking the LLM to evaluate a code submission.
    fn build_evaluation_prompt(&self, code: &str, quest_title: &str, language: &str) -> String;

    /// Build a prompt for a personalised hint.
    fn build_hint_prompt(&self, concept: &str, student_level: i32) -> String;

    /// Heuristic score when no LLM is available (stub mode).
    fn stub_score(&self, submission: &str) -> f64 {
        let lines = submission.lines().filter(|l| !l.trim().is_empty()).count();
        (lines as f64 / 15.0).clamp(0.1, 1.0)
    }
}
