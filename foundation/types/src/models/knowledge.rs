//! In-memory knowledge graph – a projection of mastery events.
//! Deterministic: same events → same graph state.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    pub concept: String,
    /// Mastery 0.0 (unseen) → 1.0 (fully mastered).
    pub mastery: f64,
    pub connected_to: Vec<String>,
    pub biome_slug: String,
    pub last_updated: Option<String>,
}

/// Student's personal knowledge graph.
/// Stored as JSONB in `students.knowledge_map` and reconstructed on demand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub student_id: Uuid,
    pub nodes: Vec<KnowledgeNode>,
    /// Weighted average of all node mastery values.
    pub overall_mastery: f64,
    /// Concepts with mastery < 0.3 → Swamp of Confusion zones.
    pub weak_concepts: Vec<String>,
    /// Concepts with mastery ≥ 0.9 → Enlightenment Peaks.
    pub mastered_concepts: Vec<String>,
}

impl KnowledgeGraph {
    pub fn new(student_id: Uuid) -> Self {
        Self { student_id, nodes: vec![], overall_mastery: 0.0, weak_concepts: vec![], mastered_concepts: vec![] }
    }

    pub fn add_concept(&mut self, concept: &str, biome_slug: &str) {
        if !self.nodes.iter().any(|n| n.concept == concept) {
            self.nodes.push(KnowledgeNode {
                concept: concept.to_string(), mastery: 0.0,
                connected_to: vec![], biome_slug: biome_slug.to_string(), last_updated: None,
            });
        }
    }

    pub fn connect(&mut self, from: &str, to: &str) {
        if let Some(n) = self.nodes.iter_mut().find(|n| n.concept == from) {
            if !n.connected_to.contains(&to.to_string()) { n.connected_to.push(to.to_string()); }
        }
    }

    /// Update mastery and recompute projections. Deterministic.
    pub fn update_mastery(&mut self, concept: &str, mastery: f64) {
        let ts = chrono::Utc::now().to_rfc3339();
        if let Some(n) = self.nodes.iter_mut().find(|n| n.concept == concept) {
            n.mastery = mastery.clamp(0.0, 1.0);
            n.last_updated = Some(ts);
        }
        self.recalc();
    }

    fn recalc(&mut self) {
        if self.nodes.is_empty() { self.overall_mastery = 0.0; return; }
        self.overall_mastery = self.nodes.iter().map(|n| n.mastery).sum::<f64>() / self.nodes.len() as f64;
        self.weak_concepts     = self.nodes.iter().filter(|n| n.mastery < 0.3).map(|n| n.concept.clone()).collect();
        self.mastered_concepts = self.nodes.iter().filter(|n| n.mastery >= 0.9).map(|n| n.concept.clone()).collect();
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    pub fn from_json(student_id: Uuid, v: &serde_json::Value) -> Self {
        serde_json::from_value(v.clone()).unwrap_or_else(|_| Self::new(student_id))
    }
}
