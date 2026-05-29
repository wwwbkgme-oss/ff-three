//! GOAP-style goal planner.
//!
//! Pure function — no mutations, no I/O, no randomness.
//! Every `GoalId` is derived deterministically via UUIDv5 from
//! `(char_id, tick, discriminant)` — required for replay and cross-node verification.

use uuid::Uuid;

use types::{CharacterId, GoalId, TickContext, WorldTick};

use crate::{
    character::Character,
    goals::{Goal, GoalStack, GoalType},
    schedule::ScheduledActivity,
};

const GOAL_ID_NS: Uuid = Uuid::from_u128(0x6b_a7b810_9dad_11d1_80b4_00c04f_d430c8);

pub struct Planner;

impl Planner {
    /// Return goals to inject into `character`'s goal stack this tick.
    pub fn suggest(character: &Character, ctx: &TickContext) -> Vec<Goal> {
        let tick = ctx.tick;
        let mut goals: Vec<Goal> = Vec::new();

        // 1. Survival overrides
        if character.stats.health <= 10
            && !Self::has_kind(&character.goals, &GoalType::RecoverHealth)
        {
            goals.push(Self::make(GoalType::RecoverHealth, 100, character.id, tick, "recover_health"));
        }
        if character.stats.is_hungry() && !Self::has_kind(&character.goals, &GoalType::Eat) {
            goals.push(Self::make(GoalType::Eat, 90, character.id, tick, "eat"));
        }
        if character.stats.is_tired() && !Self::has_kind(&character.goals, &GoalType::Sleep) {
            goals.push(Self::make(GoalType::Sleep, 85, character.id, tick, "sleep"));
        }

        // 2. Schedule-driven
        let sched = character.schedule.activity_at(tick);
        if let Some((kind, priority)) = Self::sched_goal(sched) {
            if !Self::has_kind(&character.goals, &kind) {
                goals.push(Self::make(kind, priority, character.id, tick, "schedule"));
            }
        }

        goals
    }

    fn has_kind(stack: &GoalStack, kind: &GoalType) -> bool {
        stack.active.as_ref().map(|g| &g.kind == kind).unwrap_or(false)
            || stack.pending.iter().any(|g| &g.kind == kind)
    }

    fn sched_goal(activity: ScheduledActivity) -> Option<(GoalType, i32)> {
        match activity {
            ScheduledActivity::Sleep => Some((GoalType::Sleep, 80)),
            ScheduledActivity::Eat   => Some((GoalType::Eat,   75)),
            ScheduledActivity::Work  => Some((GoalType::Work,  65)),
            _                        => None,
        }
    }

    fn make(kind: GoalType, priority: i32, char_id: CharacterId, tick: WorldTick, disc: &str) -> Goal {
        Goal {
            id:            Self::derive_id(char_id, tick, disc),
            kind,
            priority:      priority.clamp(0, 100),
            preconditions: vec![],
            deadline:      None,
            created_at:    tick,
        }
    }

    fn derive_id(char_id: CharacterId, tick: WorldTick, disc: &str) -> GoalId {
        let mut seed = Vec::with_capacity(16 + 8 + disc.len());
        seed.extend_from_slice(char_id.inner().as_bytes());
        seed.extend_from_slice(&tick.0.to_le_bytes());
        seed.extend_from_slice(disc.as_bytes());
        GoalId::from(Uuid::new_v5(&GOAL_ID_NS, &seed))
    }
}
