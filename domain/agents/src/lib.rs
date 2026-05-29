//! `agents` domain – pure agent strategies, no I/O.
//!
//! Each struct implements `AgentStrategy` from `foundation/types`.
//! Prompt strings are passed to the LLM by the runtime layer.

pub mod assessment;
pub mod curriculum;
pub mod mentor;
pub mod orchestrator;

pub use orchestrator::Orchestrator;
