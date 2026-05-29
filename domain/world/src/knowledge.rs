//! Knowledge graph domain operations.
//!
//! These functions wrap the foundation `KnowledgeGraph` type with domain rules.

use chrono::Utc;
use uuid::Uuid;

use events::AcademyEvent;
use types::KnowledgeGraph;

pub struct KnowledgeGraphOps;

impl KnowledgeGraphOps {
    /// Apply a mastery update and emit the event.
    /// Clamps mastery to [0.0, 1.0]; ignores no-ops within epsilon.
    pub fn update_mastery(
        graph:   &mut KnowledgeGraph,
        concept: &str,
        mastery: f64,
    ) -> Option<AcademyEvent> {
        let clamped = mastery.clamp(0.0, 1.0);
        let existing = graph.nodes.iter().find(|n| n.concept == concept).map(|n| n.mastery);

        if let Some(prev) = existing {
            if (prev - clamped).abs() < 1e-6 { return None; }
        } else {
            // Concept not yet in graph – add it first.
            graph.add_concept(concept, "unknown");
        }

        graph.update_mastery(concept, clamped);

        Some(AcademyEvent::ConceptMasteryUpdated {
            student_id: graph.student_id,
            concept: concept.to_owned(),
            mastery: clamped,
            timestamp: Utc::now(),
        })
    }

    /// Return concept names that are below mastery threshold (Swamp of Confusion).
    pub fn swamp_concepts(graph: &KnowledgeGraph, threshold: f64) -> Vec<String> {
        graph.nodes.iter()
            .filter(|n| n.mastery < threshold)
            .map(|n| n.concept.clone())
            .collect()
    }

    /// Return concept names that exceed the mastery threshold (Enlightenment Peaks).
    pub fn peak_concepts(graph: &KnowledgeGraph, threshold: f64) -> Vec<String> {
        graph.nodes.iter()
            .filter(|n| n.mastery >= threshold)
            .map(|n| n.concept.clone())
            .collect()
    }

    /// Add a concept from a biome if it is not already tracked.
    pub fn seed_from_biome(graph: &mut KnowledgeGraph, concepts: &[&str], biome_slug: &str) {
        for c in concepts {
            graph.add_concept(c, biome_slug);
        }
    }
}
