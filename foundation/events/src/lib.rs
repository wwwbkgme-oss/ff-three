//! `events` – foundation layer.
//!
//! **Events are truth. State is projection.**
//!
//! Every event in the system MUST be wrapped in `EventEnvelope`.

pub mod academy;
pub mod character;
pub mod commands;
pub mod envelope;

pub use academy::AcademyEvent;
pub use character::{
    CharacterEvent, CharacterKind, ConversationOutcome,
    MoodKind, SerializedEpisode, SerializedGoal,
};
pub use commands::AcademyCommand;
pub use envelope::{AcademyEnvelope, CharacterEnvelope, EventEnvelope};
