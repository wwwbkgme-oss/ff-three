//! `basic_npc` — quick-start example for the `characters` crate.
//!
//! Demonstrates:
//! - Creating an NPC with `Character::new_npc`.
//! - Issuing commands via the `AggregateRoot` command handler.
//! - Applying returned events through `CharacterReducer`.
//! - Running 5 simulation ticks with the `TickEngine`.
//!
//! Run with:
//! ```
//! cargo run --example basic_npc -p characters
//! ```

use characters::{
    commands::CharacterCommand,
    goals::GoalType,
    reducer::CharacterReducer,
    tick::TickEngine,
};
use events::ConversationOutcome;
use types::{
    traits::{AggregateRoot, CommandContext, Reducer},
    CharacterId, LocationId, WorldTick,
};

fn main() {
    // ── Create the NPC ─────────────────────────────────────────────────────────
    let npc_id   = CharacterId::new();
    let start_loc = LocationId::new();
    let mut npc = characters::character::Character::new_npc(
        npc_id, "Aldric the Baker", start_loc, WorldTick::ZERO,
    );

    println!("=== basic_npc example ===");
    println!("NPC:      {} ({})", npc.name, npc.id);
    println!("Location: {}", npc.location);
    println!("Stats:    health={} energy={} hunger={} fatigue={}\n",
             npc.stats.health, npc.stats.energy, npc.stats.hunger, npc.stats.fatigue);

    // ── Assign a custom quest goal ─────────────────────────────────────────────
    let ctx = CommandContext::test_context(WorldTick(1));
    let goal = characters::goals::Goal::new(
        GoalType::CompleteQuest("deliver_bread".to_string()), 70, WorldTick(1),
    );
    let events = npc.handle(CharacterCommand::AssignGoal { goal }, &ctx).unwrap();
    for e in &events { npc = CharacterReducer::apply(npc, e); }

    println!("After AssignGoal:");
    println!("  pending goals: {}", npc.goals.pending.len());
    println!("  active goal:   {:?}\n", npc.goals.active.as_ref().map(|g| &g.kind));

    // ── Run 5 ticks ───────────────────────────────────────────────────────────
    println!("Simulating 5 ticks (starting at tick 700 = work-hours start):");
    for t in 700u64..705 {
        let tick = WorldTick(t);
        let tick_events = TickEngine::tick(&npc, tick);
        println!("  tick {:4}: {} events emitted", t, tick_events.len());
        for e in &tick_events { npc = CharacterReducer::apply(npc, e); }
    }

    println!();
    println!("Final state:");
    println!("  health={} energy={} hunger={} fatigue={}",
             npc.stats.health, npc.stats.energy, npc.stats.hunger, npc.stats.fatigue);
    println!("  activity: {:?}", npc.activity);
    println!("  active goal: {:?}", npc.goals.active.as_ref().map(|g| &g.kind));
    println!("  pending goals: {}", npc.goals.pending.len());

    // ── Social interaction ────────────────────────────────────────────────────
    let ctx2 = CommandContext::test_context(WorldTick(800));
    let stranger = CharacterId::new();

    println!("\nConversation with stranger {}:", stranger);
    let start_events = npc
        .handle(CharacterCommand::StartConversation { with: stranger }, &ctx2)
        .unwrap();
    for e in &start_events { npc = CharacterReducer::apply(npc, e); }
    println!("  activity: {:?}", npc.activity);

    let ctx3 = CommandContext::test_context(WorldTick(810));
    let end_events = npc
        .handle(
            CharacterCommand::EndConversation {
                with:    stranger,
                outcome: ConversationOutcome::Friendly,
            },
            &ctx3,
        )
        .unwrap();
    for e in &end_events { npc = CharacterReducer::apply(npc, e); }

    if let Some(rel) = npc.relationships.get(stranger) {
        println!("  relationship: trust={:.2} affinity={:.2}", rel.trust, rel.affinity);
    }
    println!("  activity: {:?}", npc.activity);
}
