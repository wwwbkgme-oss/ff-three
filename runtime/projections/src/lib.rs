//! `projections` – event-sourced read models.
//!
//! Projections consume the event log and maintain queryable views.
//! They are read-only with respect to the event store and can be
//! rebuilt at any time by replaying from global_offset = 0.
//!
//! ## Pattern
//!
//! ```text
//! EventStore::load_since(checkpoint)
//!   └─► [StoredEvent]
//!         └─► Projection::apply(&mut self, event)
//!               └─► QueryableView  ──►  API / UI
//! ```

pub mod character_view;

pub use character_view::CharacterView;
