//! Per-tick event emission — the schedule engine.
//!
//! `TickEngine::tick` is the single entry-point called once per simulation step
//! for each character.  It reads the character's current state and returns the
//! `CharacterEvent`s that must be applied to advance it by one tick.
//!
//! **Rules (enforced by this module):**
//! - No side effects, no I/O, no wall-clock time.
//! - Same `(character, tick)` → same events, always.
//! - The caller applies the returned events via `CharacterReducer`.
//!
//! ## Ordering within one tick
//!
//! 1. **Passive stat decay** — `StatsUpdated` (hunger/fatigue accumulate,
//!    energy drains).  Skipped when all deltas would be zero.
//! 2. **Goal injection** — `GoalAdded` for any planner-suggested goals not
//!    already on the stack.
//! 3. **Goal activation** — `GoalActivated` for the highest-priority pending
//!    goal whose preconditions are currently met (post-stat-decay state is
//!    used for precondition checking, as the caller applies events in order).

use events::{CharacterEvent, SerializedGoal};
use types::WorldTick;

use crate::{character::Character, planner::Planner, schedule::ScheduledActivity};

// ── Passive stat constants ─────────────────────────────────────────────────────
//
// These are intentionally small so that a single tick has minimal observable
// impact; interesting behaviour emerges over hundreds of ticks.

/// Hunger increase per tick while awake.
const HUNGER_PER_TICK: i32 = 1;
/// Fatigue increase per tick while awake and active.
const FATIGUE_PER_TICK: i32 = 1;
/// Energy loss per tick while awake and active.
const ENERGY_PER_TICK: i32 = -1;

/// Fatigue recovery per tick while sleeping (negative = decreases fatigue).
const SLEEP_FATIGUE_RECOVERY: i32 = -5;
/// Energy recovery per tick while sleeping.
const SLEEP_ENERGY_RECOVERY: i32 = 3;

/// Hunger reduction per tick while eating.
const EAT_HUNGER_REDUCTION: i32 = -8;

/// Social-need reduction per tick while socialising.
const SOCIAL_RECOVERY_PER_TICK: i32 = -2;

/// Stateless tick-processing engine.
pub struct TickEngine;

impl TickEngine {
    /// Process one simulation tick for `character` at time `tick`.
    ///
    /// Returns the events emitted this tick.  The caller **must** apply them
    /// through `CharacterReducer` before calling `tick` again.
    pub fn tick(character: &Character, tick: WorldTick) -> Vec<CharacterEvent> {
        let mut events: Vec<CharacterEvent> = Vec::new();

        // ── 1. Passive stat decay ─────────────────────────────────────────────
        let (health_d, energy_d, hunger_d, fatigue_d, social_d) =
            Self::passive_deltas(character, tick);

        if health_d != 0
            || energy_d != 0
            || hunger_d != 0
            || fatigue_d != 0
            || social_d != 0
        {
            events.push(CharacterEvent::StatsUpdated {
                character_id:  character.id,
                health_delta:  health_d,
                energy_delta:  energy_d,
                hunger_delta:  hunger_d,
                fatigue_delta: fatigue_d,
                social_delta:  social_d,
                at:            tick,
            });
        }

        // ── 2. Goal injection ─────────────────────────────────────────────────
        for goal in Planner::suggest(character, tick) {
            events.push(CharacterEvent::GoalAdded {
                character_id: character.id,
                goal:         SerializedGoal {
                    id:       goal.id,
                    kind:     goal.kind.display_name().to_string(),
                    priority: goal.priority,
                    deadline: goal.deadline,
                },
                at: tick,
            });
        }

        // ── 3. Goal activation ────────────────────────────────────────────────
        // Activate the highest-priority pending goal whose preconditions are
        // satisfied — but only if no goal is already active.
        if character.goals.active.is_none() {
            if let Some(next) = character
                .goals
                .pending
                .iter()
                .find(|g| g.preconditions_met(&character.stats) && !g.is_expired(tick))
            {
                events.push(CharacterEvent::GoalActivated {
                    character_id: character.id,
                    goal_id:      next.id,
                    at:           tick,
                });
            }
        }

        events
    }

    /// Compute passive stat deltas for one tick given the character's schedule.
    ///
    /// Returns `(health, energy, hunger, fatigue, social)` — all signed.
    fn passive_deltas(
        character: &Character,
        tick: WorldTick,
    ) -> (i32, i32, i32, i32, i32) {
        let activity = character.schedule.activity_at(tick);
        match activity {
            ScheduledActivity::Sleep => (
                0,
                SLEEP_ENERGY_RECOVERY,
                0,                      // hunger paused while sleeping
                SLEEP_FATIGUE_RECOVERY,
                0,
            ),
            ScheduledActivity::Eat => (
                0,
                0,
                EAT_HUNGER_REDUCTION,
                FATIGUE_PER_TICK,       // eating doesn't rest the character
                0,
            ),
            ScheduledActivity::Social => (
                0,
                ENERGY_PER_TICK,
                HUNGER_PER_TICK,
                FATIGUE_PER_TICK,
                SOCIAL_RECOVERY_PER_TICK,
            ),
            // Work / Leisure / Commute / Custom all follow the baseline decay.
            _ => (
                0,
                ENERGY_PER_TICK,
                HUNGER_PER_TICK,
                FATIGUE_PER_TICK,
                0,
            ),
        }
    }
}
