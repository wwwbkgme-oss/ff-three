//! HTTP error type – maps domain/DB errors to appropriate status codes.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use types::ForgeError;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Cache error: {0}")]
    Cache(redis::RedisError),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Convenience alias for route handler return types.
pub type ServerResult<T> = Result<T, ServerError>;

impl From<ForgeError> for ServerError {
    fn from(e: ForgeError) -> Self {
        match e {
            ForgeError::NotFound { entity, id } => ServerError::NotFound(format!("{entity} '{id}' not found")),
            ForgeError::Conflict(msg)           => ServerError::Conflict(msg),
            ForgeError::DomainViolation(msg)    => ServerError::BadRequest(msg),
            ForgeError::BadRequest(msg)         => ServerError::BadRequest(msg),
            ForgeError::Unauthorised            => ServerError::Unauthorized,
        }
    }
}

impl From<redis::RedisError> for ServerError {
    fn from(e: redis::RedisError) -> Self { ServerError::Cache(e) }
}

impl From<serde_json::Error> for ServerError {
    fn from(e: serde_json::Error) -> Self {
        ServerError::Internal(anyhow::Error::from(e))
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            ServerError::NotFound(m)  => (StatusCode::NOT_FOUND,            m.clone()),
            ServerError::Conflict(m)  => (StatusCode::CONFLICT,             m.clone()),
            ServerError::BadRequest(m)=> (StatusCode::BAD_REQUEST,          m.clone()),
            ServerError::Unauthorized => (StatusCode::UNAUTHORIZED,         "Unauthorized".to_string()),
            ServerError::Database(e)  => { tracing::error!(%e, "db error"); (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into()) }
            ServerError::Cache(e)     => { tracing::error!(%e, "cache err"); (StatusCode::INTERNAL_SERVER_ERROR, "Cache error".into()) }
            ServerError::Internal(e)  => { tracing::error!(?e, "internal"); (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".into()) }
        };
        (status, Json(json!({ "error": msg, "status": status.as_u16() }))).into_response()
    }
}

// ── DB-layer error type ───────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum DbError {
    #[error("{entity} '{id}' not found")]
    NotFound { entity: &'static str, id: String },
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Database: {0}")]
    Sqlx(#[from] sqlx::Error),
}

pub type DbResult<T> = Result<T, DbError>;

impl From<DbError> for ServerError {
    fn from(e: DbError) -> Self {
        match e {
            DbError::NotFound { entity, id } => ServerError::NotFound(format!("{entity} '{id}' not found")),
            DbError::Conflict(m) => ServerError::Conflict(m),
            DbError::BadRequest(m) => ServerError::BadRequest(m),
            DbError::Sqlx(e) => ServerError::Database(e),
        }
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

pub(crate) fn not_found<T>(
    r: Result<T, sqlx::Error>, entity: &'static str, id: &str,
) -> DbResult<T> {
    r.map_err(|e| match e {
        sqlx::Error::RowNotFound => DbError::NotFound { entity, id: id.to_owned() },
        other => DbError::Sqlx(other),
    })
}

pub(crate) fn unique_err<T>(r: Result<T, sqlx::Error>, msg: &str) -> DbResult<T> {
    r.map_err(|e| match e {
        sqlx::Error::Database(ref db) if db.code().as_deref() == Some("23505") =>
            DbError::Conflict(msg.to_owned()),
        other => DbError::Sqlx(other),
    })
}
