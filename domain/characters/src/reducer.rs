//! `CharacterReducer` and the `AggregateRoot for Character` implementation.
//!
//! **State projection** (`apply`) and **command handling** (`handle`) live
//! here.  Both are pure deterministic functions — no I/O, no clock, no RNG.
//!
//! ## Architecture
//!
//! ```text
//! CharacterCommand
//!     ↓  Character::handle()   (validate + decide events)
//! Vec<CharacterEvent>
//!     ↓  CharacterReducer::apply()  (fold events into state)
//! Character  (new state)
//! ```

use events::{
    CharacterEvent, CharacterKind as EvtKind, ConversationOutcome, MoodKind,
};
use types::traits::{AggregateRoot, CommandContext, Reducer};

use crate::{
    character::{Activity, Character, CharacterKind},
    commands::CharacterCommand,
    errors::CharacterError,
    goals::{Goal, GoalType},
    memory::Episode,
    stats::Mood,
};

// ── CharacterReducer ──────────────────────────────────────────────────────────

/// Thin wrapper that satisfies the generic `Reducer<Character, CharacterEvent>` contract.
pub struct CharacterReducer;

impl Reducer<Character, CharacterEvent> for CharacterReducer {
    fn apply(state: Character, event: &CharacterEvent) -> Character {
        Character::apply(state, event)
    }
}

// ── AggregateRoot for Character ───────────────────────────────────────────────

impl AggregateRoot for Character {
    type Event   = CharacterEvent;
    type Command = CharacterCommand;
    type Error   = CharacterError;

    // ── Command handler ───────────────────────────────────────────────────────

    fn handle(
        &self,
        command: CharacterCommand,
        ctx:     &CommandContext,
    ) -> Result<Vec<CharacterEvent>, CharacterError> {
        let tick = ctx.tick;

        match command {
            // ── Movement ──────────────────────────────────────────────────────
            CharacterCommand::Move { to } => {
                if to == self.location {
                    return Err(CharacterError::violation("character is already at that location"));
                }
                Ok(vec![CharacterEvent::Moved {
                    id:   self.id,
                    from: self.location,
                    to,
                    at:   tick,
                }])
            }

            // ── Goals ─────────────────────────────────────────────────────────
            CharacterCommand::AssignGoal { goal } => {
                if self.goals.pending.len() >= crate::goals::MAX_GOALS {
                    return Err(CharacterError::GoalStackOverflow { max: crate::goals::MAX_GOALS });
                }
                use events::SerializedGoal;
                Ok(vec![CharacterEvent::GoalAdded {
                    character_id: self.id,
                    goal:         SerializedGoal {
                        id:       goal.id,
                        kind:     goal.kind.display_name().to_string(),
                        priority: goal.priority,
                        deadline: goal.deadline,
                    },
                    at: tick,
                }])
            }

            CharacterCommand::CompleteActiveGoal => {
                let goal_id = self.goals.active
                    .as_ref()
                    .map(|g| g.id)
                    .ok_or_else(|| CharacterError::violation("no active goal to complete"))?;
                Ok(vec![CharacterEvent::GoalCompleted { character_id: self.id, goal_id, at: tick }])
            }

            CharacterCommand::AbandonGoal { goal_id, reason } => {
                let exists = self.goals.active.as_ref().map(|g| g.id) == Some(goal_id)
                    || self.goals.pending.iter().any(|g| g.id == goal_id);
                if !exists {
                    return Err(CharacterError::GoalNotFound(goal_id));
                }
                Ok(vec![CharacterEvent::GoalAbandoned {
                    character_id: self.id, goal_id, reason, at: tick,
                }])
            }

            // ── Social ────────────────────────────────────────────────────────
            CharacterCommand::StartConversation { with } => {
                if with == self.id {
                    return Err(CharacterError::violation("character cannot converse with itself"));
                }
                Ok(vec![CharacterEvent::ConversationStarted {
                    initiator: self.id, partner: with, at: tick,
                }])
            }

            CharacterCommand::EndConversation { with, outcome } => {
                let (trust_d, affinity_d) = relationship_deltas_from_outcome(&outcome);
                let mut events = vec![
                    CharacterEvent::ConversationEnded {
                        initiator: self.id, partner: with, outcome, at: tick,
                    },
                    CharacterEvent::RelationshipUpdated {
                        from:           self.id,
                        to:             with,
                        trust_delta:    trust_d,
                        affinity_delta: affinity_d,
                        at:             tick,
                    },
                ];
                // Socialising reduces social need.
                events.push(CharacterEvent::StatsUpdated {
                    character_id:  self.id,
                    health_delta:  0,
                    energy_delta:  0,
                    hunger_delta:  0,
                    fatigue_delta: 0,
                    social_delta:  -15,
                    at:            tick,
                });
                Ok(events)
            }

            // ── Memory ────────────────────────────────────────────────────────
            CharacterCommand::RecordEpisode { episode, .. } => {
                use events::SerializedEpisode;
                Ok(vec![CharacterEvent::EpisodeRecorded {
                    character_id: self.id,
                    episode:      SerializedEpisode {
                        id:          episode.id,
                        summary:     episode.summary.clone(),
                        weight:      episode.weight,
                        observed_at: episode.observed_at,
                    },
                }])
            }

            CharacterCommand::ApplyDecay => {
                Ok(vec![CharacterEvent::MemoryDecayApplied { character_id: self.id, at: tick }])
            }

            // ── Factions ──────────────────────────────────────────────────────
            CharacterCommand::JoinFaction { faction_id } => {
                if self.faction == Some(faction_id) {
                    return Err(CharacterError::violation("character already in this faction"));
                }
                Ok(vec![CharacterEvent::JoinedFaction { character_id: self.id, faction_id, at: tick }])
            }

            CharacterCommand::LeaveFaction { faction_id, reason } => {
                if self.faction != Some(faction_id) {
                    return Err(CharacterError::violation("character is not in that faction"));
                }
                Ok(vec![CharacterEvent::LeftFaction { character_id: self.id, faction_id, reason, at: tick }])
            }
        }
    }

    // ── State projection ──────────────────────────────────────────────────────

    fn apply(mut state: Character, event: &CharacterEvent) -> Character {
        match event {
            // ── Lifecycle ─────────────────────────────────────────────────────
            CharacterEvent::Created { id, kind, name, location, born_at } => {
                state.id       = *id;
                state.name     = name.clone();
                state.kind     = map_kind(kind);
                state.location = *location;
                state.born_at  = *born_at;
            }
            CharacterEvent::Destroyed { .. } => {
                // Mark health = 0 so callers can detect dead characters.
                state.stats.health = 0;
            }

            // ── Movement ──────────────────────────────────────────────────────
            CharacterEvent::Moved { to, .. } => {
                state.location = *to;
                state.activity = Activity::Idle;
            }

            // ── Goals ─────────────────────────────────────────────────────────
            CharacterEvent::GoalAdded { goal, at, .. } => {
                let new_goal = Goal {
                    id:            goal.id,
                    kind:          GoalType::from_display_name(&goal.kind),
                    priority:      goal.priority,
                    preconditions: vec![],
                    deadline:      goal.deadline,
                    created_at:    *at,
                };
                // Ignore overflow silently — the planner will retry next tick.
                let _ = state.goals.push(new_goal);
            }
            CharacterEvent::GoalActivated { goal_id, .. } => {
                if let Some(pos) = state.goals.pending.iter().position(|g| g.id == *goal_id) {
                    state.goals.active = Some(state.goals.pending.remove(pos));
                }
                if let Some(goal) = &state.goals.active {
                    state.activity = Activity::ExecutingGoal(goal.id);
                }
            }
            CharacterEvent::GoalCompleted { goal_id, .. } => {
                state.goals.remove(*goal_id);
                state.activity = Activity::Idle;
            }
            CharacterEvent::GoalAbandoned { goal_id, .. } => {
                state.goals.remove(*goal_id);
                if state.activity == Activity::ExecutingGoal(*goal_id) {
                    state.activity = Activity::Idle;
                }
            }

            // ── Memory ────────────────────────────────────────────────────────
            CharacterEvent::EpisodeRecorded { episode, .. } => {
                const MAX_EPISODES: usize = 200;
                let ep = Episode::new(
                    episode.id,
                    &episode.summary,
                    episode.weight,
                    episode.observed_at,
                );
                state.memory.record(ep, MAX_EPISODES);
            }
            CharacterEvent::MemoryDecayApplied { at, .. } => {
                state.memory.apply_decay(*at);
            }
            CharacterEvent::EpisodeForgotten { episode_id, .. } => {
                state.memory.episodes.retain(|e| e.id != *episode_id);
            }

            // ── Social ────────────────────────────────────────────────────────
            CharacterEvent::RelationshipUpdated { to, trust_delta, affinity_delta, at, .. } => {
                state.relationships
                    .get_or_default_mut(*to)
                    .update(*trust_delta, *affinity_delta, *at);
            }
            CharacterEvent::ConversationStarted { partner, .. } => {
                state.activity = Activity::Conversing { with: *partner };
            }
            CharacterEvent::ConversationEnded { .. } => {
                state.activity = Activity::Idle;
            }

            // ── Factions ──────────────────────────────────────────────────────
            CharacterEvent::JoinedFaction { faction_id, .. } => {
                state.faction = Some(*faction_id);
            }
            CharacterEvent::LeftFaction { .. } => {
                state.faction = None;
            }

            // ── Mood / Stats ──────────────────────────────────────────────────
            CharacterEvent::MoodChanged { new_mood, .. } => {
                state.mood = map_mood(new_mood);
            }
            CharacterEvent::StatsUpdated {
                health_delta, energy_delta, hunger_delta, fatigue_delta, social_delta, ..
            } => {
                state.stats.health      += health_delta;
                state.stats.energy      += energy_delta;
                state.stats.hunger      += hunger_delta;
                state.stats.fatigue     += fatigue_delta;
                state.stats.social_need += social_delta;
                state.stats.clamp();
            }
        }
        // Every applied event advances the aggregate version by 1.
        // Pass ExpectedVersion::Exact(state.version) to EventStore::append.
        state.version += 1;
        state
    }
}

// ── Conversion helpers ────────────────────────────────────────────────────────

/// Map the foundation-layer `CharacterKind` to the domain-layer variant.
fn map_kind(k: &EvtKind) -> CharacterKind {
    match k {
        EvtKind::Player    => CharacterKind::Player,
        EvtKind::Npc       => CharacterKind::Npc,
        EvtKind::Agent     => CharacterKind::Agent,
        EvtKind::Companion => CharacterKind::Companion,
    }
}

/// Map `MoodKind` (foundation) to `Mood` (domain).
fn map_mood(m: &MoodKind) -> Mood {
    match m {
        MoodKind::Calm       => Mood::Calm,
        MoodKind::Happy      => Mood::Happy,
        MoodKind::Anxious    => Mood::Anxious,
        MoodKind::Angry      => Mood::Angry,
        MoodKind::Sad        => Mood::Sad,
        MoodKind::Fearful    => Mood::Fearful,
        MoodKind::Determined => Mood::Determined,
    }
}

/// Determine the trust/affinity deltas that result from a conversation outcome.
///
/// A future enhancement can factor in the character's mood via
/// `Mood::social_weight()`; for now base deltas are used.
fn relationship_deltas_from_outcome(outcome: &ConversationOutcome) -> (f32, f32) {
    match outcome {
        ConversationOutcome::Friendly                    => (0.05, 0.05),
        ConversationOutcome::Neutral                     => (0.01, 0.0),
        ConversationOutcome::Hostile                     => (-0.1, -0.1),
        ConversationOutcome::QuestAssigned               => (0.03, 0.02),
        ConversationOutcome::InformationShared { .. }    => (0.02, 0.03),
    }
}
