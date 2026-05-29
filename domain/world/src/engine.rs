//! Biome state engine – deterministic score→state mapping.

use chrono::Utc;
use uuid::Uuid;

use events::AcademyEvent;
use types::BiomeState;

pub struct BiomeStateEngine;

impl BiomeStateEngine {
    /// Convert an average assessment score to a biome state (pure, replay-safe).
    pub fn score_to_state(avg: f64) -> BiomeState {
        match avg {
            s if s < 0.30 => BiomeState::Confused,
            s if s < 0.60 => BiomeState::Clouded,
            s if s < 0.85 => BiomeState::Enlightened,
            _             => BiomeState::Mastered,
        }
    }

    /// Derive biome state from a slice of recent scores and emit the event if changed.
    pub fn recalculate(
        biome_id:      Uuid,
        current_state: &BiomeState,
        scores:        &[f64],
    ) -> Option<AcademyEvent> {
        let avg = if scores.is_empty() {
            return None;
        } else {
            scores.iter().copied().sum::<f64>() / scores.len() as f64
        };

        let new_state = Self::score_to_state(avg);
        if &new_state == current_state {
            return None;
        }

        Some(AcademyEvent::BiomeStateChanged {
            biome_id,
            new_state,
            avg_score: avg,
            timestamp: Utc::now(),
        })
    }
}
