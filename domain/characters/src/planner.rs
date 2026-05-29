//! GOAP-style goal planner.
//!
//! The planner reads character state and returns goals to inject into the goal
//! stack this tick.  It is a **pure function** — no mutations, no I/O, no
//! randomness.
//!
//! ## Goal selection priority
//!
//! 1. Critical survival (health ≤ 10 / hunger ≥ threshold / fatigue ≥ threshold)
//! 2. Schedule-driven activity (derived from `character.schedule`)
//! 3. Nothing: return an empty `Vec` (caller leaves the goal stack as-is)
//!
//! ## Determinism guarantee
//!
//! Every `GoalId` is derived deterministically via UUIDv5:
//!
//! ```text
//! id = Uuid::new_v5(GOAL_ID_NS, char_id_bytes ++ tick_le_bytes ++ discriminant_bytes)
//! ```
//!
//! This means the same `(character, tick)` always produces the same ids —
//! required for event-log replay and cross-node verification.

use uuid::Uuid;

use types::{CharacterId, GoalId, WorldTick};

use crate::{
    character::Character,
    goals::{Goal, GoalStack, GoalType},
    schedule::ScheduledActivity,
};

/// Fixed namespace for all planner-generated goal IDs.
/// Chosen as a stable constant — must never change after first deployment.
const GOAL_ID_NS: Uuid = Uuid::from_u128(0x6b_a7b810_9dad_11d1_80b4_00c04f_d430c8);

/// Stateless GOAP-style goal planner.
pub struct Planner;

impl Planner {
    /// Return goals that should be injected into `character`'s goal stack at `tick`.
    ///
    /// Callers are responsible for pushing the returned goals through
    /// `CharacterCommandHandler` (which emits `GoalAdded` events) and then
    /// applying those events via `CharacterReducer`.
    ///
    /// Returns at most a handful of goals per tick — the caller must respect
    /// `GoalStack::MAX_GOALS`.
    pub fn suggest(character: &Character, tick: WorldTick) -> Vec<Goal> {
        let mut goals: Vec<Goal> = Vec::new();

        // ── 1. Survival overrides ─────────────────────────────────────────────
        if character.stats.health <= 10
            && !Self::stack_has_kind(&character.goals, &GoalType::RecoverHealth)
        {
            goals.push(Self::make_goal(
                GoalType::RecoverHealth,
                100,
                character.id,
                tick,
                "recover_health",
            ));
        }

        if character.stats.is_hungry()
            && !Self::stack_has_kind(&character.goals, &GoalType::Eat)
        {
            goals.push(Self::make_goal(
                GoalType::Eat,
                90,
                character.id,
                tick,
                "eat",
            ));
        }

        if character.stats.is_tired()
            && !Self::stack_has_kind(&character.goals, &GoalType::Sleep)
        {
            goals.push(Self::make_goal(
                GoalType::Sleep,
                85,
                character.id,
                tick,
                "sleep",
            ));
        }

        // ── 2. Schedule-driven activity ───────────────────────────────────────
        let scheduled = character.schedule.activity_at(tick);
        if let Some((kind, priority)) = Self::goal_for_scheduled_activity(scheduled) {
            if !Self::stack_has_kind(&character.goals, &kind) {
                goals.push(Self::make_goal(kind, priority, character.id, tick, "schedule"));
            }
        }

        goals
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// `true` when the goal stack (active or pending) already contains a goal
    /// of the exact same `GoalType` variant.  Uses `PartialEq`.
    fn stack_has_kind(stack: &GoalStack, kind: &GoalType) -> bool {
        stack.active.as_ref().map(|g| &g.kind == kind).unwrap_or(false)
            || stack.pending.iter().any(|g| &g.kind == kind)
    }

    /// Map a scheduled activity to a `(GoalType, priority)` pair.
    ///
    /// Returns `None` for activities that are handled elsewhere (e.g. Social is
    /// driven by the relationship engine, not the planner).
    fn goal_for_scheduled_activity(activity: ScheduledActivity) -> Option<(GoalType, i32)> {
        match activity {
            ScheduledActivity::Sleep   => Some((GoalType::Sleep, 80)),
            ScheduledActivity::Eat     => Some((GoalType::Eat,   75)),
            ScheduledActivity::Work    => Some((GoalType::Work,  65)),
            // Social / Leisure / Commute / Custom → handled outside the planner.
            _ => None,
        }
    }

    /// Build a `Goal` whose `id` is derived deterministically from context.
    ///
    /// The `discriminant` string distinguishes goals of the same kind emitted
    /// by different planner rules (e.g. `"eat"` vs `"schedule"`).
    fn make_goal(
        kind:         GoalType,
        priority:     i32,
        char_id:      CharacterId,
        tick:         WorldTick,
        discriminant: &str,
    ) -> Goal {
        let id = Self::derive_id(char_id, tick, discriminant);
        Goal {
            id,
            kind,
            priority: priority.clamp(0, 100),
            preconditions: vec![],
            deadline: None,
            created_at: tick,
        }
    }

    /// Derive a `GoalId` deterministically from `(char_id, tick, discriminant)`.
    ///
    /// Uses UUIDv5 so the output is stable across process restarts and nodes.
    fn derive_id(char_id: CharacterId, tick: WorldTick, discriminant: &str) -> GoalId {
        let mut seed: Vec<u8> =
            Vec::with_capacity(16 + 8 + discriminant.len());
        seed.extend_from_slice(char_id.inner().as_bytes());
        seed.extend_from_slice(&tick.0.to_le_bytes());
        seed.extend_from_slice(discriminant.as_bytes());
        GoalId::from(Uuid::new_v5(&GOAL_ID_NS, &seed))
    }
}
