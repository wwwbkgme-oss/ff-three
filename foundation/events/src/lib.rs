//! `events` – foundation layer.
//!
//! All events and commands that flow through the Academy system.
//! **Events are truth. State is projection.**

pub mod academy;
pub mod commands;

pub use academy::AcademyEvent;
pub use commands::AcademyCommand;
