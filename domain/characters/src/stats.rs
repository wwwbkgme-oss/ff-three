//! Character statistics and mood.

use serde::{Deserialize, Serialize};

/// Numeric stats representing the character's physical and social state.
///
/// All values are kept within their declared ranges by the reducer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub health:      i32,  // 0..max_health
    pub max_health:  i32,
    pub energy:      i32,  // 0..max_energy
    pub max_energy:  i32,
    /// 0 = full, 100 = starving.
    pub hunger:      i32,
    /// 0 = rested, 100 = exhausted.
    pub fatigue:     i32,
    /// 0 = socially satisfied, 100 = lonely.
    pub social_need: i32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            health:      100,
            max_health:  100,
            energy:      100,
            max_energy:  100,
            hunger:      0,
            fatigue:     0,
            social_need: 0,
        }
    }
}

impl Stats {
    /// Clamp all values to their legal ranges (called after every reducer step).
    pub fn clamp(&mut self) {
        self.health      = self.health.clamp(0, self.max_health);
        self.energy      = self.energy.clamp(0, self.max_energy);
        self.hunger      = self.hunger.clamp(0, 100);
        self.fatigue     = self.fatigue.clamp(0, 100);
        self.social_need = self.social_need.clamp(0, 100);
    }

    pub fn is_alive(&self) -> bool { self.health > 0 }
    pub fn is_hungry(&self)  -> bool { self.hunger  > 60 }
    pub fn is_tired(&self)   -> bool { self.fatigue > 70 }
    pub fn is_lonely(&self)  -> bool { self.social_need > 70 }
}

/// Emotional state of the character — influences goal selection and social reactions.
///
/// Transitions are deterministic: driven by `MoodChanged` events, not probability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Calm,
    Happy,
    Anxious,
    Angry,
    Sad,
    Fearful,
    Determined,
}

impl Default for Mood {
    fn default() -> Self { Self::Calm }
}

impl Mood {
    /// A rough social weight for this mood (-1.0..1.0).
    /// Used when computing relationship deltas during conversations.
    pub fn social_weight(&self) -> f32 {
        match self {
            Mood::Happy     =>  0.3,
            Mood::Calm      =>  0.1,
            Mood::Determined=>  0.2,
            Mood::Sad       => -0.1,
            Mood::Anxious   => -0.1,
            Mood::Fearful   => -0.2,
            Mood::Angry     => -0.4,
        }
    }
}
