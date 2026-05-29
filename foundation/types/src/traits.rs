//! Foundation trait contracts.
//!
//! These traits define the INTERFACES that domain and runtime crates implement.
//! They contain no I/O — all implementations must stay within their allowed layer.

use crate::{
    ids::{ActorId, CorrelationId, EventId, RealmId},
    time::WorldTick,
};

// ── Event-sourcing contracts ──────────────────────────────────────────────────

/// Apply a single event to a state, producing the next state.
///
/// **Invariants (must hold for all implementations):**
///   - Deterministic: same `(state, event)` → same output, always.
///   - Side-effect free: no I/O, no randomness, no wall-clock time.
///   - Associative: sequential application matches batch application.
pub trait Reducer<S, E> {
    fn apply(state: S, event: &E) -> S;
}

/// Immutable context injected into every command handler.
///
/// Provides deterministic time and trace metadata without exposing runtime I/O.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current simulation tick — use this instead of `Utc::now()`.
    pub tick:           WorldTick,
    /// Who (player, agent, system) issued the command.
    pub actor:          ActorId,
    /// Which world-shard this command targets.
    pub realm:          RealmId,
    /// Trace chain for this logical operation.
    pub correlation_id: CorrelationId,
    /// The event that directly caused this command, if any.
    pub causation_id:   Option<EventId>,
}

impl CommandContext {
    /// Minimal constructor for tests and single-realm deployments.
    pub fn test_context(tick: WorldTick) -> Self {
        Self {
            tick,
            actor:          ActorId::new(),
            realm:          RealmId::new(),
            correlation_id: CorrelationId::new(),
            causation_id:   None,
        }
    }
}

/// Handle a command by validating it against current state and emitting events.
///
/// **Separation from `Reducer`:**
///   - `CommandHandler` validates and decides **which** events to emit.
///   - `Reducer::apply` deterministically applies those events to state.
///
/// **Implementations MUST be:**
///   - Deterministic (same inputs → same output).
///   - Side-effect free.
pub trait CommandHandler<S, C, E> {
    type Error;
    fn handle(&self, state: &S, command: C, ctx: &CommandContext) -> Result<Vec<E>, Self::Error>;
}

/// A domain aggregate root — combines command handling and state projection.
///
/// This is the recommended interface for complex aggregates (Character, Quest, etc.).
/// Separates command validation (business rules) from state projection (reducers).
///
/// ```text
/// Command → AggregateRoot::handle() → [Events]
///           [Events] → Reducer::apply() → NewState
/// ```
pub trait AggregateRoot: Sized {
    /// Domain event type emitted by this aggregate.
    type Event;
    /// Command type accepted by this aggregate.
    type Command;
    /// Domain error type.
    type Error;

    /// Validate `command` against current `state` and emit zero or more events.
    ///
    /// Must be deterministic and side-effect free.
    fn handle(
        &self,
        command: Self::Command,
        ctx:     &CommandContext,
    ) -> Result<Vec<Self::Event>, Self::Error>;

    /// Apply a single event, producing the next state.
    fn apply(state: Self, event: &Self::Event) -> Self;

    /// Replay a sequence of events from an initial state.
    fn replay(initial: Self, events: &[Self::Event]) -> Self {
        events.iter().fold(initial, |s, e| Self::apply(s, e))
    }
}

// ── Agent strategy contract ───────────────────────────────────────────────────

/// Pure, synchronous strategy contract for all Academy agents.
///
/// Methods return prompts or computed values — they do NOT perform I/O.
/// The runtime layer supplies those prompts to LLM endpoints.
pub trait AgentStrategy: Send + Sync {
    fn name(&self) -> &str;
    fn adapt_difficulty(&self, recent_scores: &[f64]) -> i32;
    fn needs_mentor(&self, recent_scores: &[f64]) -> bool;
    fn build_quest_prompt(&self, goal: &str, biome: &str, difficulty: i32) -> String;
    fn build_evaluation_prompt(&self, code: &str, quest_title: &str, language: &str) -> String;
    fn build_hint_prompt(&self, concept: &str, student_level: i32) -> String;
    fn stub_score(&self, submission: &str) -> f64 {
        (submission.lines().filter(|l| !l.trim().is_empty()).count() as f64 / 15.0).clamp(0.1, 1.0)
    }
}
