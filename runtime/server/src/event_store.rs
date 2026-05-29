//! Postgres-backed `EventStore` implementation.
//!
//! Uses `sqlx` with the `event_streams` + `events` tables from migration 005.
//!
//! ## Optimistic concurrency
//!
//! `append` runs inside a transaction with `SELECT ... FOR UPDATE` on
//! `event_streams` so two concurrent writers for the same stream are
//! serialised — only the first succeeds.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use server::event_store::PgEventStore;
//! use events::{EventStore, ExpectedVersion, StreamId};
//!
//! let store = PgEventStore::new(pool.clone());
//! let new_version = store.append(
//!     StreamId::from_uuid(character.id.inner()),
//!     ExpectedVersion::Exact(character.version),
//!     serialised_events,
//! ).await?;
//! ```

use async_trait::async_trait;
use serde_json::Value;
use sqlx::PgPool;
use events::{EventStore, ExpectedVersion, StoredEvent, StreamId};

/// Postgres-backed event store.
#[derive(Clone)]
pub struct PgEventStore {
    pool: PgPool,
}

impl PgEventStore {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[derive(Debug, thiserror::Error)]
pub enum PgStoreError {
    #[error("concurrency conflict: expected version {expected} but stream is at {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },
    #[error("stream already exists")]
    StreamAlreadyExists,
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("serialisation error: {0}")]
    Serialisation(#[from] serde_json::Error),
}

#[async_trait]
impl EventStore for PgEventStore {
    type Error = PgStoreError;

    async fn append(
        &self,
        stream_id:        StreamId,
        expected_version: ExpectedVersion,
        events:           Vec<Value>,
    ) -> Result<u64, PgStoreError> {
        if events.is_empty() {
            return self.stream_version(stream_id).await;
        }

        let mut tx = self.pool.begin().await?;

        // Upsert the stream row and lock it for the duration of the transaction.
        let current_version: i64 = sqlx::query_scalar(
            r#"
            INSERT INTO event_streams (stream_id, version)
            VALUES ($1, 0)
            ON CONFLICT (stream_id) DO UPDATE SET stream_id = EXCLUDED.stream_id
            RETURNING version
            "#,
        )
        .bind(stream_id.0)
        .fetch_one(&mut *tx)
        .await?;

        let current = current_version as u64;

        // Validate optimistic concurrency.
        match expected_version {
            ExpectedVersion::Any => {}
            ExpectedVersion::NoStream => {
                if current > 0 {
                    tx.rollback().await?;
                    return Err(PgStoreError::StreamAlreadyExists);
                }
            }
            ExpectedVersion::Exact(v) => {
                if current != v {
                    tx.rollback().await?;
                    return Err(PgStoreError::ConcurrencyConflict { expected: v, actual: current });
                }
            }
        }

        let n = events.len() as i64;

        // Bulk insert events.
        for (i, payload) in events.iter().enumerate() {
            let seq = current as i64 + i as i64;
            sqlx::query(
                "INSERT INTO events (stream_id, sequence, payload) VALUES ($1, $2, $3)",
            )
            .bind(stream_id.0)
            .bind(seq)
            .bind(payload)
            .execute(&mut *tx)
            .await?;
        }

        // Bump stream version.
        sqlx::query("UPDATE event_streams SET version = version + $1 WHERE stream_id = $2")
            .bind(n)
            .bind(stream_id.0)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(current + n as u64)
    }

    async fn load_stream(&self, stream_id: StreamId) -> Result<Vec<StoredEvent>, PgStoreError> {
        let rows: Vec<(i64, i64, Value)> = sqlx::query_as(
            "SELECT global_offset, sequence, payload FROM events
             WHERE stream_id = $1 ORDER BY sequence ASC",
        )
        .bind(stream_id.0)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(off, seq, payload)| StoredEvent {
            stream_id,
            sequence:     seq as u64,
            global_offset: off as u64,
            payload,
        }).collect())
    }

    async fn load_since(
        &self, from_offset: u64, limit: usize,
    ) -> Result<Vec<StoredEvent>, PgStoreError> {
        let rows: Vec<(i64, uuid::Uuid, i64, Value)> = sqlx::query_as(
            "SELECT global_offset, stream_id, sequence, payload FROM events
             WHERE global_offset >= $1 ORDER BY global_offset ASC LIMIT $2",
        )
        .bind(from_offset as i64)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(off, sid, seq, payload)| StoredEvent {
            stream_id:    StreamId(sid),
            sequence:     seq as u64,
            global_offset: off as u64,
            payload,
        }).collect())
    }

    async fn stream_version(&self, stream_id: StreamId) -> Result<u64, PgStoreError> {
        let v: Option<i64> = sqlx::query_scalar(
            "SELECT version FROM event_streams WHERE stream_id = $1",
        )
        .bind(stream_id.0)
        .fetch_optional(&self.pool)
        .await?;
        Ok(v.unwrap_or(0) as u64)
    }
}
