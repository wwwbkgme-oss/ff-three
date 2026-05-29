//! Per-tick event emission — the schedule engine.
//!
//! `TickEngine::tick` is the single entry-point called once per simulation step
//! for each character.  It reads the character's current state and returns the
//! `CharacterEvent`s that must be applied to advance it by one tick.
//!
//! **Rules (enforced by this module):**
//! - No side effects, no I/O, no wall-clock time.
//! - Same `(character, ctx)` → same events, always.
//! - The caller applies the returned events via `CharacterReducer`.
//!
//! ## Ordering within one tick
//!
//! 1. **Passive stat decay** — `StatsUpdated`.  Skipped when all deltas are zero.
//! 2. **Goal injection** — `GoalAdded` for planner-suggested goals.
//! 3. **Goal activation** — `GoalActivated` for the highest-priority pending goal.

use events::{CharacterEvent, SerializedGoal};
use types::TickContext;

use crate::{character::Character, planner::Planner, schedule::ScheduledActivity};

const HUNGER_PER_TICK:        i32 = 1;
const FATIGUE_PER_TICK:       i32 = 1;
const ENERGY_PER_TICK:        i32 = -1;
const SLEEP_FATIGUE_RECOVERY: i32 = -5;
const SLEEP_ENERGY_RECOVERY:  i32 = 3;
const EAT_HUNGER_REDUCTION:   i32 = -8;
const SOCIAL_RECOVERY_PER_TICK: i32 = -2;

pub struct TickEngine;

impl TickEngine {
    /// Process one simulation tick for `character`.
    pub fn tick(character: &Character, ctx: &TickContext) -> Vec<CharacterEvent> {
        let tick = ctx.tick;
        let mut events: Vec<CharacterEvent> = Vec::new();

        // 1. Passive stat decay
        let (health_d, energy_d, hunger_d, fatigue_d, social_d) =
            Self::passive_deltas(character, ctx);

        if health_d != 0 || energy_d != 0 || hunger_d != 0 || fatigue_d != 0 || social_d != 0 {
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

        // 2. Goal injection
        for goal in Planner::suggest(character, ctx) {
            events.push(CharacterEvent::GoalAdded {
                character_id: character.id,
                goal: SerializedGoal {
                    id:       goal.id,
                    kind:     goal.kind.display_name().to_string(),
                    priority: goal.priority,
                    deadline: goal.deadline,
                },
                at: tick,
            });
        }

        // 3. Goal activation
        if character.goals.active.is_none() {
            if let Some(next) = character.goals.pending.iter()
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

    fn passive_deltas(character: &Character, ctx: &TickContext) -> (i32, i32, i32, i32, i32) {
        let activity = character.schedule.activity_at(ctx.tick);
        match activity {
            ScheduledActivity::Sleep  => (0, SLEEP_ENERGY_RECOVERY, 0, SLEEP_FATIGUE_RECOVERY, 0),
            ScheduledActivity::Eat    => (0, 0, EAT_HUNGER_REDUCTION, FATIGUE_PER_TICK, 0),
            ScheduledActivity::Social => (0, ENERGY_PER_TICK, HUNGER_PER_TICK, FATIGUE_PER_TICK, SOCIAL_RECOVERY_PER_TICK),
            _                         => (0, ENERGY_PER_TICK, HUNGER_PER_TICK, FATIGUE_PER_TICK, 0),
        }
    }
}
