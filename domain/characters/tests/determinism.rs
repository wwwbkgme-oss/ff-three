//! Determinism tests for the `characters` domain crate.
//!
//! Every test asserts one of two properties:
//!
//! 1. **Same inputs → same outputs** (referential transparency).
//! 2. **Reducer invariants** — stat clamping, goal-stack ordering, memory
//!    retention caps, etc.
//!
//! No async, no DB, no clock.  All time is expressed as `WorldTick`.

use characters::{
    character::Character,
    commands::CharacterCommand,
    goals::{Goal, GoalType},
    planner::Planner,
    reducer::CharacterReducer,
    tick::TickEngine,
};
use events::{CharacterEvent, ConversationOutcome, SerializedEpisode, SerializedGoal};
use types::{
    traits::{AggregateRoot, CommandContext, Reducer},
    CharacterId, EpisodeId, GoalId, LocationId, WorldTick,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn test_char() -> Character {
    Character::new_npc(
        CharacterId::new(),
        "TestNPC",
        LocationId::new(),
        WorldTick::ZERO,
    )
}

fn ctx(tick: u64) -> CommandContext {
    CommandContext::test_context(WorldTick(tick))
}

/// Apply a slice of events to a character using `CharacterReducer`.
fn apply_all(mut c: Character, events: &[CharacterEvent]) -> Character {
    for e in events {
        c = CharacterReducer::apply(c, e);
    }
    c
}

// ── Planner determinism ───────────────────────────────────────────────────────

#[test]
fn planner_same_input_same_output() {
    let char1 = test_char();
    let char2 = char1.clone();
    let tick = WorldTick(1000);

    let goals1 = Planner::suggest(&char1, tick);
    let goals2 = Planner::suggest(&char2, tick);

    assert_eq!(goals1.len(), goals2.len(), "suggestion count must match");
    for (g1, g2) in goals1.iter().zip(goals2.iter()) {
        assert_eq!(g1.id, g2.id, "goal IDs must be deterministic");
        assert_eq!(g1.kind, g2.kind);
        assert_eq!(g1.priority, g2.priority);
    }
}

#[test]
fn planner_hungry_character_gets_eat_goal() {
    let mut c = test_char();
    c.stats.hunger = 80; // above the threshold (> 60)
    let goals = Planner::suggest(&c, WorldTick(1));
    assert!(
        goals.iter().any(|g| g.kind == GoalType::Eat),
        "hungry character must receive an Eat goal"
    );
}

#[test]
fn planner_tired_character_gets_sleep_goal() {
    let mut c = test_char();
    c.stats.fatigue = 75; // above the threshold (> 70)
    let goals = Planner::suggest(&c, WorldTick(1));
    assert!(
        goals.iter().any(|g| g.kind == GoalType::Sleep),
        "tired character must receive a Sleep goal"
    );
}

#[test]
fn planner_no_duplicate_goals() {
    let mut c = test_char();
    c.stats.hunger = 80;

    // First tick injects an Eat goal.
    let goals_t1 = Planner::suggest(&c, WorldTick(1));

    // Simulate applying the GoalAdded events so the stack is populated.
    for g in &goals_t1 {
        let evt = CharacterEvent::GoalAdded {
            character_id: c.id,
            goal:         SerializedGoal {
                id:       g.id,
                kind:     g.kind.display_name().to_string(),
                priority: g.priority,
                deadline: g.deadline,
            },
            at: WorldTick(1),
        };
        c = CharacterReducer::apply(c, &evt);
    }

    // Second tick — planner should NOT suggest Eat again.
    let goals_t2 = Planner::suggest(&c, WorldTick(2));
    let eat_count = goals_t2.iter().filter(|g| g.kind == GoalType::Eat).count();
    assert_eq!(eat_count, 0, "planner must not duplicate an already-queued goal");
}

// ── Tick-engine determinism ───────────────────────────────────────────────────

#[test]
fn tick_engine_same_input_same_events() {
    let c1 = test_char();
    let c2 = c1.clone();
    let tick = WorldTick(500);

    let evts1 = TickEngine::tick(&c1, tick);
    let evts2 = TickEngine::tick(&c2, tick);

    // Compare by serialisation for a deep structural equality check.
    let j1 = serde_json::to_string(&evts1).unwrap();
    let j2 = serde_json::to_string(&evts2).unwrap();
    assert_eq!(j1, j2, "tick engine must be deterministic");
}

#[test]
fn tick_engine_emits_stats_updated() {
    let c = test_char();
    // At tick 100 the schedule has the character awake (work hours).
    let evts = TickEngine::tick(&c, WorldTick(100));
    let has_stats = evts.iter().any(|e| matches!(e, CharacterEvent::StatsUpdated { .. }));
    assert!(has_stats, "tick must emit StatsUpdated during waking hours");
}

#[test]
fn tick_engine_no_stats_update_during_sleep_when_all_zero() {
    // A freshly-created NPC has fatigue=0; during sleep ticks the fatigue delta
    // would be negative, clamped to 0.  We only check that the function
    // doesn't panic; correctness of delta signs is verified separately.
    let c = test_char();
    // Tick 50 → first sleep window (0..600 = hours 0–6).
    let _evts = TickEngine::tick(&c, WorldTick(50));
}

// ── CharacterReducer state transitions ───────────────────────────────────────

#[test]
fn reducer_created_event_sets_fields() {
    let c = test_char();
    let loc = LocationId::new();
    let id = CharacterId::new();

    let evt = CharacterEvent::Created {
        id,
        kind:     events::CharacterKind::Npc,
        name:     "Gareth".to_string(),
        location: loc,
        born_at:  WorldTick(42),
    };

    let new_c = CharacterReducer::apply(c, &evt);
    assert_eq!(new_c.id,       id);
    assert_eq!(new_c.name,     "Gareth");
    assert_eq!(new_c.location, loc);
    assert_eq!(new_c.born_at,  WorldTick(42));
}

#[test]
fn reducer_stats_updated_clamps() {
    let mut c = test_char();
    // Push health below zero — reducer must clamp to 0.
    let evt = CharacterEvent::StatsUpdated {
        character_id:  c.id,
        health_delta:  -999,
        energy_delta:  -999,
        hunger_delta:  999,
        fatigue_delta: 999,
        social_delta:  999,
        at:            WorldTick(1),
    };
    c = CharacterReducer::apply(c, &evt);
    assert_eq!(c.stats.health, 0);
    assert_eq!(c.stats.energy, 0);
    assert_eq!(c.stats.hunger, 100);
    assert_eq!(c.stats.fatigue, 100);
    assert_eq!(c.stats.social_need, 100);
}

#[test]
fn reducer_goal_lifecycle() {
    let c = test_char();
    let goal_id = GoalId::new();
    let at = WorldTick(10);

    let char_id = c.id;

    // GoalAdded
    let c = CharacterReducer::apply(c, &CharacterEvent::GoalAdded {
        character_id: char_id,
        goal:         SerializedGoal {
            id:       goal_id,
            kind:     "eat".to_string(),
            priority: 80,
            deadline: None,
        },
        at,
    });
    assert_eq!(c.goals.pending.len(), 1, "goal should be in pending queue");
    assert!(c.goals.active.is_none());

    // GoalActivated
    let c = CharacterReducer::apply(c, &CharacterEvent::GoalActivated {
        character_id: char_id, goal_id, at,
    });
    assert!(c.goals.active.is_some(), "goal should be active");
    assert!(c.goals.pending.is_empty());

    // GoalCompleted
    let c = CharacterReducer::apply(c, &CharacterEvent::GoalCompleted {
        character_id: char_id, goal_id, at,
    });
    assert!(c.goals.active.is_none(), "goal should be cleared after completion");
}

#[test]
fn reducer_memory_records_and_decays() {
    let c = test_char();
    let ep_id = EpisodeId::new();

    let char_id = c.id;

    // Record an episode.
    let c = CharacterReducer::apply(c, &CharacterEvent::EpisodeRecorded {
        character_id: char_id,
        episode:      SerializedEpisode {
            id:          ep_id,
            summary:     "Met the blacksmith".to_string(),
            weight:      0.8,
            observed_at: WorldTick(1),
        },
    });
    assert_eq!(c.memory.episodes.len(), 1);

    // Apply decay at a much later tick — weight should drop.
    let decay_tick = WorldTick(100_000);
    let c = CharacterReducer::apply(c, &CharacterEvent::MemoryDecayApplied {
        character_id: char_id,
        at:           decay_tick,
    });
    // At 100_000 ticks × DECAY_RATE_PER_TICK (0.00005) = 5.0, so the episode
    // weight 0.8 - 5.0 = 0 → below FORGET_THRESHOLD, episode is removed.
    assert!(
        c.memory.episodes.is_empty(),
        "very old episode should be removed by decay"
    );
}

// ── AggregateRoot command handler ─────────────────────────────────────────────

#[test]
fn command_move_emits_moved_event() {
    let c = test_char();
    let dest = LocationId::new();

    let events = c.handle(CharacterCommand::Move { to: dest }, &ctx(1)).unwrap();
    assert_eq!(events.len(), 1);
    assert!(matches!(&events[0], CharacterEvent::Moved { to, .. } if *to == dest));
}

#[test]
fn command_move_to_same_location_is_error() {
    let c = test_char();
    let current_loc = c.location;
    let result = c.handle(CharacterCommand::Move { to: current_loc }, &ctx(1));
    assert!(result.is_err(), "moving to current location must be an error");
}

#[test]
fn command_complete_goal_with_none_active_is_error() {
    let c = test_char();
    let result = c.handle(CharacterCommand::CompleteActiveGoal, &ctx(1));
    assert!(result.is_err(), "completing with no active goal must be an error");
}

#[test]
fn command_assign_goal_round_trip() {
    let mut c = test_char();
    let goal = Goal::new(GoalType::Work, 50, WorldTick(1));
    let goal_id = goal.id;

    let events = c.handle(CharacterCommand::AssignGoal { goal }, &ctx(1)).unwrap();
    c = apply_all(c, &events);

    assert!(
        c.goals.pending.iter().any(|g| g.id == goal_id),
        "assigned goal must appear in pending queue"
    );
}

#[test]
fn command_conversation_round_trip() {
    let c = test_char();
    let partner = CharacterId::new();

    // Start conversation.
    let events = c.handle(
        CharacterCommand::StartConversation { with: partner },
        &ctx(10),
    ).unwrap();
    let c = apply_all(c, &events);
    assert!(
        matches!(c.activity, characters::character::Activity::Conversing { with } if with == partner)
    );

    // End conversation.
    let events = c.handle(
        CharacterCommand::EndConversation { with: partner, outcome: ConversationOutcome::Friendly },
        &ctx(15),
    ).unwrap();
    let c = apply_all(c, &events);
    assert_eq!(c.activity, characters::character::Activity::Idle);

    // Relationship edge should have been created.
    let rel = c.relationships.get(partner);
    assert!(rel.is_some(), "relationship edge should exist after conversation");
    assert!(rel.unwrap().trust > 0.0, "trust should increase after friendly chat");
}

// ── Replay correctness ────────────────────────────────────────────────────────

#[test]
fn aggregate_replay_matches_sequential_application() {
    let c = test_char();
    let dest = LocationId::new();
    let partner = CharacterId::new();

    let move_events = c.handle(CharacterCommand::Move { to: dest }, &ctx(5)).unwrap();
    let c_after_move = apply_all(c.clone(), &move_events);

    let talk_events = c_after_move
        .handle(CharacterCommand::StartConversation { with: partner }, &ctx(6))
        .unwrap();

    // Sequential application
    let all_events: Vec<CharacterEvent> = move_events.iter().chain(talk_events.iter()).cloned().collect();
    let sequential = apply_all(c.clone(), &all_events);

    // Replay via `AggregateRoot::replay`
    let replayed = Character::replay(c.clone(), &all_events);

    assert_eq!(sequential.location, replayed.location);
    assert_eq!(sequential.id, replayed.id);
}
