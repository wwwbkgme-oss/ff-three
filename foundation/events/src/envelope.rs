//! EventEnvelope — wraps every emitted event with causal and trace metadata.
//!
//! Rule: **no naked event is ever emitted**.
//! All emissions go through `EventEnvelope<E>`.

use serde::{Deserialize, Serialize};

use types::{
    ActorId, CorrelationId, EventId, RealmId, WorldTick,
};

/// Universal event wrapper.
///
/// Every event emitted anywhere in the system MUST be wrapped in this envelope.
/// This enables:
///   - Causality tracing (`causation_id`)
///   - Distributed correlation (`correlation_id`)
///   - Replay debugging (`event_id`, `tick`)
///   - Schema migration (`schema_version`)
///   - Multi-realm sync (`realm`, `actor`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<E> {
    /// Globally unique identifier for deduplication and replay.
    pub event_id:       EventId,
    /// The event that directly caused this one, if any.
    pub causation_id:   Option<EventId>,
    /// Trace ID linking a causal chain of events into one logical operation.
    pub correlation_id: CorrelationId,
    /// Schema version of `payload` — bump when the event structure changes.
    pub schema_version: u32,
    /// Simulation tick at emission time.  Never wall-clock time.
    pub tick:           WorldTick,
    /// The agent, player, or system that triggered this event.
    pub actor:          ActorId,
    /// The world shard or realm this event belongs to.
    pub realm:          RealmId,
    pub payload:        E,
}

impl<E: Clone> EventEnvelope<E> {
    /// Convenience constructor for tests and single-realm deployments.
    pub fn test(payload: E, tick: WorldTick) -> Self {
        Self {
            event_id:       EventId::new(),
            causation_id:   None,
            correlation_id: CorrelationId::new(),
            schema_version: 1,
            tick,
            actor:          ActorId::new(),
            realm:          RealmId::new(),
            payload,
        }
    }
}

/// Convenience type alias for Academy events.
pub type AcademyEnvelope = EventEnvelope<super::academy::AcademyEvent>;

/// Convenience type alias for Character events.
pub type CharacterEnvelope = EventEnvelope<super::character::CharacterEvent>;
