//! Quest command handler – the Single Mutation Path for quest state.
//!
//! Takes a command + context, validates domain rules, and emits events.
//! No I/O; the runtime layer persists the emitted events.

use chrono::Utc;
use uuid::Uuid;

use events::AcademyEvent;
use types::{
    AchievementType, ForgeError, ForgeResult, Quest, QuestStatus,
    Student, StudentQuest, level_for_xp,
};

use super::rules::{MAX_ATTEMPTS, PASS_THRESHOLD};

/// Everything the handler needs to make a decision – injected by runtime.
pub struct QuestContext<'a> {
    pub student:               &'a Student,
    pub quest:                 &'a Quest,
    pub student_quest:         Option<&'a StudentQuest>,
    pub completed_quest_count: i64,
}

/// Stateless command handler for the quest domain.
pub struct QuestCommandHandler;

impl QuestCommandHandler {
    /// Emit events for starting a quest.
    pub fn start<'a>(&self, ctx: &QuestContext<'a>) -> ForgeResult<Vec<AcademyEvent>> {
        if ctx.quest.status == QuestStatus::Locked {
            return Err(ForgeError::DomainViolation("Quest is locked".into()));
        }

        let attempts = ctx.student_quest.map(|sq| sq.attempts).unwrap_or(0);
        if attempts >= MAX_ATTEMPTS {
            return Err(ForgeError::DomainViolation(
                format!("Maximum attempts ({MAX_ATTEMPTS}) reached for this quest"),
            ));
        }

        Ok(vec![AcademyEvent::QuestStarted {
            student_id: ctx.student.id,
            quest_id:   ctx.quest.id,
            attempt:    attempts + 1,
            timestamp:  Utc::now(),
        }])
    }

    /// Emit events for completing a quest after scoring.
    pub fn complete<'a>(
        &self,
        ctx:   &QuestContext<'a>,
        score: f64,
    ) -> ForgeResult<Vec<AcademyEvent>> {
        let passed  = score >= PASS_THRESHOLD;
        let attempt = ctx.student_quest.map(|sq| sq.attempts).unwrap_or(1);
        let mut evts: Vec<AcademyEvent> = Vec::new();

        if passed {
            let new_xp    = ctx.student.xp + ctx.quest.xp_reward;
            let new_level = level_for_xp(new_xp);

            evts.push(AcademyEvent::QuestCompleted {
                student_id: ctx.student.id,
                quest_id:   ctx.quest.id,
                score,
                xp_awarded: ctx.quest.xp_reward,
                timestamp:  Utc::now(),
            });
            evts.push(AcademyEvent::XpGained {
                student_id: ctx.student.id,
                amount:     ctx.quest.xp_reward,
                new_total:  new_xp,
                timestamp:  Utc::now(),
            });

            if new_level > ctx.student.level {
                evts.push(AcademyEvent::LevelUp { student_id: ctx.student.id, new_level, timestamp: Utc::now() });
            }

            // First quest ever completed.
            if ctx.completed_quest_count == 0 {
                evts.push(achievement(ctx.student.id, AchievementType::FirstBlood,   "First Blood",   10));
            }
            // Perfect score.
            if (score - 1.0).abs() < f64::EPSILON {
                evts.push(achievement(ctx.student.id, AchievementType::Perfectionist, "Perfectionist", 25));
            }
            evts.push(achievement(ctx.student.id, AchievementType::QuestCompleted,
                &format!("Quest Completed: {}", ctx.quest.title), 5));
        } else {
            evts.push(AcademyEvent::QuestFailed {
                student_id: ctx.student.id,
                quest_id:   ctx.quest.id,
                score,
                attempt,
                timestamp:  Utc::now(),
            });
        }

        Ok(evts)
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn achievement(sid: Uuid, t: AchievementType, title: &str, xp: i32) -> AcademyEvent {
    AcademyEvent::AchievementEarned {
        student_id:      sid,
        achievement_type: t,
        title:           title.to_owned(),
        xp_reward:       xp,
        timestamp:       Utc::now(),
    }
}
