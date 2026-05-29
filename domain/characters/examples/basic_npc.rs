//! `basic_npc` — quick-start example for the `characters` crate.
//!
//! Run with: cargo run --example basic_npc -p characters

use characters::{
    commands::CharacterCommand,
    goals::GoalType,
    reducer::CharacterReducer,
    tick::TickEngine,
};
use events::ConversationOutcome;
use types::{
    traits::{AggregateRoot, CommandContext, Reducer},
    CharacterId, LocationId, TickContext, WorldTick,
};

fn main() {
    let mut npc = characters::character::Character::new_npc(
        CharacterId::new(), "Aldric the Baker", LocationId::new(), WorldTick::ZERO,
    );

    println!("=== basic_npc ===");
    println!("NPC: {} ({})", npc.name, npc.id);
    println!("Stats: health={} energy={} hunger={} fatigue={}\n",
             npc.stats.health, npc.stats.energy, npc.stats.hunger, npc.stats.fatigue);

    // Assign a quest goal
    let cctx = CommandContext::test_context(WorldTick(1));
    let goal = characters::goals::Goal::new(
        GoalType::CompleteQuest("deliver_bread".into()), 70, WorldTick(1),
    );
    let evs = npc.handle(CharacterCommand::AssignGoal { goal }, &cctx).unwrap();
    for e in &evs { npc = CharacterReducer::apply(npc, e); }
    println!("After AssignGoal: pending={} active={:?}\n",
             npc.goals.pending.len(), npc.goals.active.as_ref().map(|g| &g.kind));

    // Run 5 ticks
    println!("5 ticks starting at tick 700 (work hours):");
    for t in 700u64..705 {
        let tctx = TickContext::test(t);
        let evs = TickEngine::tick(&npc, &tctx);
        println!("  tick {t}: {} events", evs.len());
        for e in &evs { npc = CharacterReducer::apply(npc, e); }
    }
    println!("\nFinal: health={} energy={} hunger={} fatigue={} version={}",
             npc.stats.health, npc.stats.energy, npc.stats.hunger, npc.stats.fatigue, npc.version);
    println!("activity: {:?}", npc.activity);

    // Conversation
    let partner = CharacterId::new();
    let cctx2 = CommandContext::test_context(WorldTick(800));
    println!("\nConversation with {}:", partner);
    let evs = npc.handle(CharacterCommand::StartConversation { with: partner }, &cctx2).unwrap();
    for e in &evs { npc = CharacterReducer::apply(npc, e); }
    println!("  activity: {:?}", npc.activity);

    let cctx3 = CommandContext::test_context(WorldTick(810));
    let evs = npc.handle(CharacterCommand::EndConversation {
        with: partner, outcome: ConversationOutcome::Friendly,
    }, &cctx3).unwrap();
    for e in &evs { npc = CharacterReducer::apply(npc, e); }
    if let Some(r) = npc.relationships.get(partner) {
        println!("  trust={:.2} affinity={:.2}", r.trust, r.affinity);
    }
}
