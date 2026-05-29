//! Social relationship graph.
//!
//! Relationships are the emergent social fabric between characters.
//! All mutations arrive as `RelationshipUpdated` events — never direct mutation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use types::{CharacterId, WorldTick};

/// Directed, weighted relationship from one character to another.
///
/// `trust`  and `affinity` range from -1.0 (hostile) to +1.0 (deep bond).
/// `familiarity` accumulates from 0.0 (strangers) to 1.0 (intimate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Trust: willingness to act in the target's interest.
    pub trust:             f32,
    /// Affinity: emotional warmth or dislike.
    pub affinity:          f32,
    /// Familiarity: how well they know each other.
    pub familiarity:       f32,
    pub last_interacted:   Option<WorldTick>,
    pub interaction_count: u32,
}

impl Default for Relationship {
    fn default() -> Self {
        Self {
            trust:             0.0,
            affinity:          0.0,
            familiarity:       0.0,
            last_interacted:   None,
            interaction_count: 0,
        }
    }
}

impl Relationship {
    /// Apply trust and affinity deltas, clamping to [-1.0, 1.0].
    pub fn update(&mut self, trust_delta: f32, affinity_delta: f32, at: WorldTick) {
        self.trust     = (self.trust     + trust_delta).clamp(-1.0, 1.0);
        self.affinity  = (self.affinity  + affinity_delta).clamp(-1.0, 1.0);
        self.familiarity = (self.familiarity + 0.01).clamp(0.0, 1.0);
        self.last_interacted   = Some(at);
        self.interaction_count += 1;
    }

    pub fn is_hostile(&self)  -> bool { self.trust < -0.5 }
    pub fn is_friendly(&self) -> bool { self.trust  > 0.4 && self.affinity > 0.4 }
    pub fn is_stranger(&self) -> bool { self.familiarity < 0.1 }
}

/// All outgoing relationships from one character.
///
/// Uses `BTreeMap` so iteration order is deterministic — required for
/// `DeterministicHash` computation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipGraph {
    pub edges: BTreeMap<CharacterId, Relationship>,
}

impl RelationshipGraph {
    /// Get or create a relationship entry.
    pub fn get_or_default_mut(&mut self, target: CharacterId) -> &mut Relationship {
        self.edges.entry(target).or_default()
    }

    /// Get the relationship to `target`, returning `None` if unknown.
    pub fn get(&self, target: CharacterId) -> Option<&Relationship> {
        self.edges.get(&target)
    }

    /// All characters above the given trust threshold.
    pub fn allies(&self, trust_threshold: f32) -> Vec<CharacterId> {
        self.edges.iter()
            .filter(|(_, r)| r.trust >= trust_threshold)
            .map(|(id, _)| *id)
            .collect()
    }

    /// All characters below the given trust threshold.
    pub fn enemies(&self, trust_threshold: f32) -> Vec<CharacterId> {
        self.edges.iter()
            .filter(|(_, r)| r.trust <= trust_threshold)
            .map(|(id, _)| *id)
            .collect()
    }
}
