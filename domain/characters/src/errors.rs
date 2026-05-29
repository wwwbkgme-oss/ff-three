//! Domain errors for the character system.

use thiserror::Error;

use types::{CharacterId, GoalId};

#[derive(Error, Debug)]
pub enum CharacterError {
    #[error("Character {0} not found")]
    NotFound(CharacterId),

    #[error("Goal {0} not found")]
    GoalNotFound(GoalId),

    #[error("Goal stack overflow (max {max} goals reached)")]
    GoalStackOverflow { max: usize },

    #[error("Domain rule violated: {0}")]
    DomainViolation(String),

    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
}

impl CharacterError {
    pub fn violation(msg: impl Into<String>) -> Self {
        Self::DomainViolation(msg.into())
    }
}
