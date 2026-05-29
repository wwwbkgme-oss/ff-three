//! Event store contract and in-memory reference implementation.
//!
//! The `EventStore` trait is the **only** interface through which domain
//! aggregates are persisted.  Nothing in `domain/` writes to a database
//! directly — it emits events, and the runtime layer appends them here.
//!
//! ## Three core operations
//!
//! | Method            | Purpose                                        |
//! |-------------------|------------------------------------------------|
//! | `append`          | Write events, enforce optimistic concurrency   |
//! | `load_stream`     | Replay all events for one aggregate            |
//! | `load_since`      | Fan-out from a global offset (projections)     |
//!
//! ## Optimistic concurrency
//!
//! Every `append` validates `ExpectedVersion`.  If the stream has been
//! written concurrently, `append` returns `StoreError::ConcurrencyConflict`
//! and the caller must retry from the current state.
//!
//! ## Implementations shipped here
//!
//! - `InMemoryEventStore` — zero-dep, single-process, for unit tests.
//! - `PgEventStore` lives in `runtime/server` (needs sqlx/Postgres).

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

// ── StreamId ──────────────────────────────────────────────────────────────────

/// Identifies the event stream for one aggregate instance.
///
/// One stream per aggregate root.  Derive from the domain ID:
/// `StreamId::from_uuid(character.id.inner())`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
         serde::Serialize, serde::Deserialize)]
pub struct StreamId(pub Uuid);

impl StreamId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn from_uuid(id: Uuid) -> Self { Self(id) }
}

impl Default for StreamId { fn default() -> Self { Self::new() } }

impl std::fmt::Display for StreamId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
}

impl From<Uuid> for StreamId { fn from(u: Uuid) -> Self { Self(u) } }

// ── ExpectedVersion ───────────────────────────────────────────────────────────

/// Caller's assertion about the current stream version before appending.
///
/// Stream version = total events ever appended (0 = empty).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedVersion {
    /// No concurrency check — last writer wins.
    Any,
    /// Stream must not exist yet (version == 0).
    NoStream,
    /// Stream must be at exactly this version.
    Exact(u64),
}

// ── StoredEvent ───────────────────────────────────────────────────────────────

/// One persisted event.
///
/// `payload` is opaque JSON.  Call `deserialize::<E>()` to recover the type.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEvent {
    pub stream_id:     StreamId,
    /// Zero-based position within this stream.
    pub sequence:      u64,
    /// Monotonically increasing position across **all** streams.
    /// Projections use this as their replay checkpoint.
    pub global_offset: u64,
    pub payload:       Value,
}

impl StoredEvent {
    pub fn deserialize<E: serde::de::DeserializeOwned>(&self) -> Result<E, serde_json::Error> {
        serde_json::from_value(self.payload.clone())
    }
}

// ── StoreError ────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("concurrency conflict: expected version {expected} but stream is at {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },

    #[error("stream already exists (ExpectedVersion::NoStream violated)")]
    StreamAlreadyExists,

    #[error("serialisation error: {0}")]
    Serialisation(#[from] serde_json::Error),
}

// ── EventStore trait ──────────────────────────────────────────────────────────

#[async_trait]
pub trait EventStore: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Append pre-serialised events to a stream.
    /// Returns the **new stream version**.
    async fn append(
        &self,
        stream_id:        StreamId,
        expected_version: ExpectedVersion,
        events:           Vec<Value>,
    ) -> Result<u64, Self::Error>;

    /// All events for one aggregate, in emission order.
    async fn load_stream(&self, stream_id: StreamId) -> Result<Vec<StoredEvent>, Self::Error>;

    /// Events with `global_offset >= from_offset`, up to `limit`.
    async fn load_since(
        &self, from_offset: u64, limit: usize,
    ) -> Result<Vec<StoredEvent>, Self::Error>;

    /// Current version (= event count) for a stream.  0 when absent.
    async fn stream_version(&self, stream_id: StreamId) -> Result<u64, Self::Error>;
}

// ── InMemoryEventStore ────────────────────────────────────────────────────────

/// Ephemeral, thread-safe event store for tests and examples.
/// Never use in production.
#[derive(Debug, Default, Clone)]
pub struct InMemoryEventStore {
    inner: Arc<RwLock<MemState>>,
}

#[derive(Debug, Default)]
struct MemState {
    streams:    BTreeMap<StreamId, Vec<StoredEvent>>,
    global_log: Vec<(StreamId, usize)>,
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    type Error = StoreError;

    async fn append(
        &self,
        stream_id:        StreamId,
        expected_version: ExpectedVersion,
        events:           Vec<Value>,
    ) -> Result<u64, StoreError> {
        let mut s = self.inner.write().unwrap();

        let current = s.streams.get(&stream_id).map(|v| v.len() as u64).unwrap_or(0);

        match expected_version {
            ExpectedVersion::Any => {}
            ExpectedVersion::NoStream => {
                if current > 0 { return Err(StoreError::StreamAlreadyExists); }
            }
            ExpectedVersion::Exact(v) => {
                if current != v {
                    return Err(StoreError::ConcurrencyConflict { expected: v, actual: current });
                }
            }
        }

        if events.is_empty() { return Ok(current); }

        let global_start = s.global_log.len() as u64;
        let n = events.len();

        let new_stored: Vec<StoredEvent> = events.into_iter().enumerate().map(|(i, p)| {
            StoredEvent {
                stream_id,
                sequence:     current + i as u64,
                global_offset: global_start + i as u64,
                payload:       p,
            }
        }).collect();

        let idx_start = current as usize;
        s.streams.entry(stream_id).or_default().extend(new_stored);
        for i in 0..n { s.global_log.push((stream_id, idx_start + i)); }

        Ok(current + n as u64)
    }

    async fn load_stream(&self, stream_id: StreamId) -> Result<Vec<StoredEvent>, StoreError> {
        Ok(self.inner.read().unwrap().streams.get(&stream_id).cloned().unwrap_or_default())
    }

    async fn load_since(&self, from_offset: u64, limit: usize) -> Result<Vec<StoredEvent>, StoreError> {
        let s = self.inner.read().unwrap();
        let result = s.global_log.iter()
            .skip(from_offset as usize)
            .take(limit)
            .filter_map(|(sid, idx)| s.streams.get(sid).and_then(|v| v.get(*idx)).cloned())
            .collect();
        Ok(result)
    }

    async fn stream_version(&self, stream_id: StreamId) -> Result<u64, StoreError> {
        Ok(self.inner.read().unwrap().streams.get(&stream_id).map(|v| v.len() as u64).unwrap_or(0))
    }
}
