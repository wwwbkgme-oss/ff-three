//! Foundation error types.
//! These represent domain-level failures, not infrastructure failures.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ForgeError {
    /// The requested entity does not exist.
    #[error("{entity} '{id}' not found")]
    NotFound { entity: &'static str, id: String },

    /// An entity with this identity already exists.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// The caller violated a domain rule.
    #[error("Domain rule violated: {0}")]
    DomainViolation(String),

    /// The input data is invalid.
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// The caller is not authorised.
    #[error("Unauthorised")]
    Unauthorised,
}

pub type ForgeResult<T> = Result<T, ForgeError>;
