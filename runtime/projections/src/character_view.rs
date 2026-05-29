//! Character read model — lightweight view for API responses.
//!
//! `CharacterView` is cheaper to load than the full `Character` aggregate:
//! no goal stack, no memory graph, no relationship graph — only what most
//! queries need.
//!
//! ## Lifecycle
//!
//! 1. Build from a full aggregate: `CharacterView::from_character(c, tick, offset)`
//! 2. Update incrementally: `view.apply(event)` — O(1) per event
//! 3. Persist `(view, checkpoint)` so restarts can resume from that offset

use serde::{Deserialize, Serialize};

use characters::character::Character;
use events::CharacterEvent;
use types::{CharacterId, LocationId, WorldTick};

/// Flattened, serialisable snapshot of a character for read-side consumers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterView {
    pub id:          CharacterId,
    pub name:        String,
    pub health:      i32,
    pub energy:      i32,
    pub hunger:      i32,
    pub fatigue:     i32,
    pub social_need: i32,
    pub location:    LocationId,
    /// Aggregate version when last updated.
    pub version:     u64,
    /// `StoredEvent::global_offset` of the last event applied.
    /// Persist this to resume after restart.
    pub checkpoint:  u64,
    pub updated_at:  WorldTick,
}

impl CharacterView {
    /// Build from a full aggregate after a replay.
    pub fn from_character(c: &Character, at: WorldTick, checkpoint: u64) -> Self {
        Self {
            id:          c.id,
            name:        c.name.clone(),
            health:      c.stats.health,
            energy:      c.stats.energy,
            hunger:      c.stats.hunger,
            fatigue:     c.stats.fatigue,
            social_need: c.stats.social_need,
            location:    c.location,
            version:     c.version,
            checkpoint,
            updated_at:  at,
        }
    }

    /// Apply one `CharacterEvent` incrementally.
    ///
    /// Returns `true` when the event was relevant to this character and the
    /// view was mutated.  Always advance `checkpoint` in the caller regardless.
    pub fn apply(&mut self, event: &CharacterEvent, global_offset: u64) -> bool {
        let relevant = match event {
            CharacterEvent::Moved { id, to, at, .. } if *id == self.id => {
                self.location   = *to;
                self.updated_at = *at;
                true
            }
            CharacterEvent::StatsUpdated {
                character_id,
                health_delta, energy_delta, hunger_delta, fatigue_delta, social_delta,
                at, ..
            } if *character_id == self.id => {
                self.health      = (self.health      + health_delta) .clamp(0, 200);
                self.energy      = (self.energy      + energy_delta) .clamp(0, 200);
                self.hunger      = (self.hunger      + hunger_delta) .clamp(0, 100);
                self.fatigue     = (self.fatigue     + fatigue_delta).clamp(0, 100);
                self.social_need = (self.social_need + social_delta) .clamp(0, 100);
                self.updated_at  = *at;
                true
            }
            _ => false,
        };
        if relevant { self.version += 1; }
        self.checkpoint = global_offset;
        relevant
    }
}
