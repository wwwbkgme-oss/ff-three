//! `world` domain – biome state machine and knowledge graph operations.
//!
//! Deterministic: same inputs always produce the same state and events.

pub mod engine;
pub mod knowledge;

pub use engine::BiomeStateEngine;
pub use knowledge::KnowledgeGraphOps;
