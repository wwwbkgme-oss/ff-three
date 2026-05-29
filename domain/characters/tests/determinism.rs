//! Determinism tests for the `characters` domain crate.
//!
//! 1. Same inputs → same outputs (referential transparency).
//! 2. Reducer invariants — stat clamping, goal-stack ordering, memory caps.

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
    CharacterId, EpisodeId, GoalId, LocationId, TickContext, WorldTick,
};

fn test_char() -> Character {
    Character::new_npc(CharacterId::new(), "TestNPC", LocationId::new(), WorldTick::ZERO)
}

fn ctx(tick: u64) -> CommandContext { CommandContext::test_context(WorldTick(tick)) }
fn tctx(tick: u64) -> TickContext   { TickContext::test(tick) }

fn apply_all(mut c: Character, events: &[CharacterEvent]) -> Character {
    for e in events { c = CharacterReducer::apply(c, e); }
    c
}

// ── Planner ───────────────────────────────────────────────────────────────────

#[test]
fn planner_same_input_same_output() {
    let char1 = test_char();
    let char2 = char1.clone();
    let ctx = tctx(1000);
    let goals1 = Planner::suggest(&char1, &ctx);
    let goals2 = Planner::suggest(&char2, &ctx);
    assert_eq!(goals1.len(), goals2.len());
    for (g1, g2) in goals1.iter().zip(goals2.iter()) {
        assert_eq!(g1.id, g2.id, "goal IDs must be deterministic");
        assert_eq!(g1.kind, g2.kind);
    }
}

#[test]
fn planner_hungry_character_gets_eat_goal() {
    let mut c = test_char();
    c.stats.hunger = 80;
    let goals = Planner::suggest(&c, &tctx(1));
    assert!(goals.iter().any(|g| g.kind == GoalType::Eat));
}

#[test]
fn planner_tired_character_gets_sleep_goal() {
    let mut c = test_char();
    c.stats.fatigue = 75;
    let goals = Planner::suggest(&c, &tctx(1));
    assert!(goals.iter().any(|g| g.kind == GoalType::Sleep));
}

#[test]
fn planner_no_duplicate_goals() {
    let mut c = test_char();
    c.stats.hunger = 80;
    let char_id = c.id;

    let goals_t1 = Planner::suggest(&c, &tctx(1));
    for g in &goals_t1 {
        c = CharacterReducer::apply(c, &CharacterEvent::GoalAdded {
            character_id: char_id,
            goal: SerializedGoal {
                id: g.id, kind: g.kind.display_name().to_string(),
                priority: g.priority, deadline: g.deadline,
            },
            at: WorldTick(1),
        });
    }
    let goals_t2 = Planner::suggest(&c, &tctx(2));
    assert_eq!(goals_t2.iter().filter(|g| g.kind == GoalType::Eat).count(), 0);
}

// ── Tick engine ───────────────────────────────────────────────────────────────

#[test]
fn tick_engine_same_input_same_events() {
    let c1 = test_char();
    let c2 = c1.clone();
    let ctx = tctx(500);
    let j1 = serde_json::to_string(&TickEngine::tick(&c1, &ctx)).unwrap();
    let j2 = serde_json::to_string(&TickEngine::tick(&c2, &ctx)).unwrap();
    assert_eq!(j1, j2, "tick engine must be deterministic");
}

#[test]
fn tick_engine_emits_stats_updated() {
    let c = test_char();
    let evts = TickEngine::tick(&c, &tctx(100));
    assert!(evts.iter().any(|e| matches!(e, CharacterEvent::StatsUpdated { .. })));
}

#[test]
fn tick_engine_no_panic_during_sleep() {
    let c = test_char();
    let _ = TickEngine::tick(&c, &tctx(50)); // tick 50 is in sleep window 0-600
}

// ── Reducer ───────────────────────────────────────────────────────────────────

#[test]
fn reducer_created_event_sets_fields() {
    let c = test_char();
    let loc = LocationId::new();
    let id  = CharacterId::new();
    let new_c = CharacterReducer::apply(c, &CharacterEvent::Created {
        id, kind: events::CharacterKind::Npc,
        name: "Gareth".to_string(), location: loc, born_at: WorldTick(42),
    });
    assert_eq!(new_c.id, id);
    assert_eq!(new_c.name, "Gareth");
    assert_eq!(new_c.location, loc);
    assert_eq!(new_c.born_at, WorldTick(42));
}

#[test]
fn reducer_stats_updated_clamps() {
    let mut c = test_char();
    let cid = c.id;
    c = CharacterReducer::apply(c, &CharacterEvent::StatsUpdated {
        character_id: cid,
        health_delta: -999, energy_delta: -999, hunger_delta: 999,
        fatigue_delta: 999, social_delta: 999, at: WorldTick(1),
    });
    assert_eq!(c.stats.health, 0);
    assert_eq!(c.stats.energy, 0);
    assert_eq!(c.stats.hunger, 100);
    assert_eq!(c.stats.fatigue, 100);
    assert_eq!(c.stats.social_need, 100);
}

#[test]
fn reducer_version_increments() {
    let c = test_char();
    assert_eq!(c.version, 0);
    let c = CharacterReducer::apply(c.clone(), &CharacterEvent::Moved {
        id: c.id, from: c.location, to: LocationId::new(), at: WorldTick(1),
    });
    assert_eq!(c.version, 1);
}

#[test]
fn reducer_goal_lifecycle() {
    let c = test_char();
    let goal_id = GoalId::new();
    let at = WorldTick(10);
    let char_id = c.id;

    let c = CharacterReducer::apply(c, &CharacterEvent::GoalAdded {
        character_id: char_id,
        goal: SerializedGoal { id: goal_id, kind: "eat".to_string(), priority: 80, deadline: None },
        at,
    });
    assert_eq!(c.goals.pending.len(), 1);
    assert!(c.goals.active.is_none());

    let c = CharacterReducer::apply(c, &CharacterEvent::GoalActivated { character_id: char_id, goal_id, at });
    assert!(c.goals.active.is_some());

    let c = CharacterReducer::apply(c, &CharacterEvent::GoalCompleted { character_id: char_id, goal_id, at });
    assert!(c.goals.active.is_none());
}

#[test]
fn reducer_memory_records_and_decays() {
    let c = test_char();
    let ep_id = EpisodeId::new();
    let char_id = c.id;

    let c = CharacterReducer::apply(c, &CharacterEvent::EpisodeRecorded {
        character_id: char_id,
        episode: SerializedEpisode {
            id: ep_id, summary: "Met the blacksmith".to_string(),
            weight: 0.8, observed_at: WorldTick(1),
        },
    });
    assert_eq!(c.memory.episodes.len(), 1);

    let c = CharacterReducer::apply(c, &CharacterEvent::MemoryDecayApplied {
        character_id: char_id, at: WorldTick(100_000),
    });
    assert!(c.memory.episodes.is_empty(), "old episode must decay");
}

// ── Commands ──────────────────────────────────────────────────────────────────

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
    let loc = c.location;
    assert!(c.handle(CharacterCommand::Move { to: loc }, &ctx(1)).is_err());
}

#[test]
fn command_complete_goal_with_none_active_is_error() {
    let c = test_char();
    assert!(c.handle(CharacterCommand::CompleteActiveGoal, &ctx(1)).is_err());
}

#[test]
fn command_assign_goal_round_trip() {
    let mut c = test_char();
    let goal = Goal::new(GoalType::Work, 50, WorldTick(1));
    let goal_id = goal.id;
    let events = c.handle(CharacterCommand::AssignGoal { goal }, &ctx(1)).unwrap();
    c = apply_all(c, &events);
    assert!(c.goals.pending.iter().any(|g| g.id == goal_id));
}

#[test]
fn command_conversation_round_trip() {
    let c = test_char();
    let partner = CharacterId::new();

    let events = c.handle(CharacterCommand::StartConversation { with: partner }, &ctx(10)).unwrap();
    let c = apply_all(c, &events);
    assert!(matches!(c.activity, characters::character::Activity::Conversing { with } if with == partner));

    let events = c.handle(
        CharacterCommand::EndConversation { with: partner, outcome: ConversationOutcome::Friendly },
        &ctx(15),
    ).unwrap();
    let c = apply_all(c, &events);
    assert_eq!(c.activity, characters::character::Activity::Idle);
    assert!(c.relationships.get(partner).map(|r| r.trust > 0.0).unwrap_or(false));
}

// ── Replay ────────────────────────────────────────────────────────────────────

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

    let all: Vec<CharacterEvent> = move_events.iter().chain(talk_events.iter()).cloned().collect();
    let sequential = apply_all(c.clone(), &all);
    let replayed   = Character::replay(c.clone(), &all);

    assert_eq!(sequential.location, replayed.location);
    assert_eq!(sequential.version,  replayed.version);
}
