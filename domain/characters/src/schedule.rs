//! Character schedule — daily and weekly routines.
//!
//! All time references use `WorldTick`, not wall-clock time.
//! `tick_of_day()` maps an absolute tick to a within-day offset.

use serde::{Deserialize, Serialize};

use types::{DAY_LENGTH_TICKS, WorldTick, tick_of_day};

/// A block of scheduled activity within a day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    /// Start tick within the day cycle (0..DAY_LENGTH_TICKS).
    pub day_start:  u64,
    /// End tick within the day cycle (exclusive).
    pub day_end:    u64,
    pub activity:   ScheduledActivity,
    /// Higher value = harder to interrupt.
    pub priority:   i32,
}

impl TimeSlot {
    pub fn contains(&self, day_tick: u64) -> bool {
        day_tick >= self.day_start && day_tick < self.day_end
    }
}

/// The scheduled activity type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledActivity {
    Sleep,
    Eat,
    Work,
    Leisure,
    Social,
    Commute,
    Custom(String),
}

/// A character's daily schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub slots: Vec<TimeSlot>,
}

impl Default for Schedule {
    fn default() -> Self { Self::default_npc() }
}

impl Schedule {
    /// A sensible default schedule for a generic NPC.
    ///
    /// DAY_LENGTH_TICKS = 2400, so each "hour" ≈ 100 ticks.
    ///
    /// ```
    /// 0000–0600  Sleep    (0–600)
    /// 0600–0700  Eat      (600–700)
    /// 0700–1200  Work     (700–1200)
    /// 1200–1300  Eat      (1200–1300)
    /// 1300–1800  Work     (1300–1800)
    /// 1800–2000  Leisure  (1800–2000)
    /// 2000–2200  Social   (2000–2200)
    /// 2200–2400  Sleep    (2200–2400)
    /// ```
    pub fn default_npc() -> Self {
        let h = |hours: u64| hours * (DAY_LENGTH_TICKS / 24);
        Self {
            slots: vec![
                TimeSlot { day_start: h(0),  day_end: h(6),  activity: ScheduledActivity::Sleep,   priority: 90 },
                TimeSlot { day_start: h(6),  day_end: h(7),  activity: ScheduledActivity::Eat,     priority: 80 },
                TimeSlot { day_start: h(7),  day_end: h(12), activity: ScheduledActivity::Work,    priority: 70 },
                TimeSlot { day_start: h(12), day_end: h(13), activity: ScheduledActivity::Eat,     priority: 80 },
                TimeSlot { day_start: h(13), day_end: h(18), activity: ScheduledActivity::Work,    priority: 70 },
                TimeSlot { day_start: h(18), day_end: h(20), activity: ScheduledActivity::Leisure, priority: 50 },
                TimeSlot { day_start: h(20), day_end: h(22), activity: ScheduledActivity::Social,  priority: 60 },
                TimeSlot { day_start: h(22), day_end: h(24), activity: ScheduledActivity::Sleep,   priority: 90 },
            ],
        }
    }

    /// Return the slot active at `tick`, if any.
    pub fn current_slot(&self, tick: WorldTick) -> Option<&TimeSlot> {
        let day_tick = tick_of_day(tick);
        self.slots.iter().find(|s| s.contains(day_tick))
    }

    /// Return the scheduled activity at `tick`, defaulting to `Leisure`.
    pub fn activity_at(&self, tick: WorldTick) -> ScheduledActivity {
        self.current_slot(tick)
            .map(|s| s.activity.clone())
            .unwrap_or(ScheduledActivity::Leisure)
    }
}
