//! `daily_schedule` — simulate a full NPC day and print hourly snapshots.
//!
//! Run with: cargo run --example daily_schedule -p characters

use characters::{character::Character, reducer::CharacterReducer, schedule::ScheduledActivity, tick::TickEngine};
use types::{traits::Reducer, CharacterId, LocationId, TickContext, WorldTick, DAY_LENGTH_TICKS};

const HOUR_TICKS: u64 = DAY_LENGTH_TICKS / 24;

fn main() {
    let mut npc = Character::new_npc(CharacterId::new(), "Marta the Guard", LocationId::new(), WorldTick::ZERO);

    println!("=== daily_schedule (DAY_LENGTH_TICKS={DAY_LENGTH_TICKS}) ===\n");
    println!("{:<6} {:<14} {:>7} {:>7} {:>8} {:>9} {:>9} {:>7} {:>8}",
             "tick","schedule","health","energy","hunger","fatigue","social","goals","version");
    println!("{}", "-".repeat(80));

    for hour in 0u64..24 {
        let tick = WorldTick(hour * HOUR_TICKS);
        let tctx = TickContext::test(tick.0);
        let evts = TickEngine::tick(&npc, &tctx);
        for e in &evts { npc = CharacterReducer::apply(npc, e); }

        let sched = npc.schedule.activity_at(tick);
        let label = match &sched {
            ScheduledActivity::Sleep   => "sleep",
            ScheduledActivity::Eat     => "eat",
            ScheduledActivity::Work    => "work",
            ScheduledActivity::Leisure => "leisure",
            ScheduledActivity::Social  => "social",
            ScheduledActivity::Commute => "commute",
            ScheduledActivity::Custom(s) => s.as_str(),
        };
        let goal_count = npc.goals.pending.len() + npc.goals.active.as_ref().map(|_| 1).unwrap_or(0);
        println!("{:<6} {:<14} {:>7} {:>7} {:>8} {:>9} {:>9} {:>7} {:>8}",
                 tick.0, label,
                 npc.stats.health, npc.stats.energy, npc.stats.hunger,
                 npc.stats.fatigue, npc.stats.social_need, goal_count, npc.version);
    }

    println!("\nEnd-of-day: alive={} hungry={} tired={} lonely={} version={}",
             npc.stats.is_alive(), npc.stats.is_hungry(), npc.stats.is_tired(),
             npc.stats.is_lonely(), npc.version);
}
