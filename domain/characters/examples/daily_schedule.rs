//! `daily_schedule` — simulate a full NPC day and print hourly snapshots.
//!
//! Demonstrates how `TickEngine` drives passive stat changes and goal
//! injection across the full `DAY_LENGTH_TICKS` cycle.
//!
//! Run with:
//! ```
//! cargo run --example daily_schedule -p characters
//! ```

use characters::{
    character::Character,
    reducer::CharacterReducer,
    schedule::ScheduledActivity,
    tick::TickEngine,
};
use types::{
    traits::Reducer,
    CharacterId, LocationId, WorldTick, DAY_LENGTH_TICKS,
};

/// One "hour" in the default schedule = DAY_LENGTH_TICKS / 24.
const HOUR_TICKS: u64 = DAY_LENGTH_TICKS / 24;

fn main() {
    let mut npc = Character::new_npc(
        CharacterId::new(),
        "Marta the Guard",
        LocationId::new(),
        WorldTick::ZERO,
    );

    println!("=== daily_schedule example ===");
    println!("NPC: {} (DAY_LENGTH_TICKS = {})\n", npc.name, DAY_LENGTH_TICKS);
    println!("{:<6} {:<14} {:>7} {:>7} {:>8} {:>9} {:>9} {:>6}",
             "tick", "schedule", "health", "energy", "hunger", "fatigue",
             "social", "goals");
    println!("{}", "-".repeat(70));

    // Simulate a full day, printing a snapshot every hour.
    for hour in 0u64..24 {
        let tick = WorldTick(hour * HOUR_TICKS);

        // Apply one tick's events.
        let events = TickEngine::tick(&npc, tick);
        for e in &events {
            npc = CharacterReducer::apply(npc, e);
        }

        let sched = npc.schedule.activity_at(tick);
        let activity_label = match &sched {
            ScheduledActivity::Sleep   => "sleep",
            ScheduledActivity::Eat     => "eat",
            ScheduledActivity::Work    => "work",
            ScheduledActivity::Leisure => "leisure",
            ScheduledActivity::Social  => "social",
            ScheduledActivity::Commute => "commute",
            ScheduledActivity::Custom(s) => s.as_str(),
        };

        println!("{:<6} {:<14} {:>7} {:>7} {:>8} {:>9} {:>9} {:>6}",
                 tick.0,
                 activity_label,
                 npc.stats.health,
                 npc.stats.energy,
                 npc.stats.hunger,
                 npc.stats.fatigue,
                 npc.stats.social_need,
                 npc.goals.pending.len()
                     + npc.goals.active.as_ref().map(|_| 1).unwrap_or(0));
    }

    println!("\nEnd-of-day summary:");
    println!("  alive:         {}", npc.stats.is_alive());
    println!("  hungry:        {}", npc.stats.is_hungry());
    println!("  tired:         {}", npc.stats.is_tired());
    println!("  lonely:        {}", npc.stats.is_lonely());
    println!("  active goal:   {:?}", npc.goals.active.as_ref().map(|g| g.kind.display_name()));
    println!("  total pending: {}", npc.goals.pending.len());
}
