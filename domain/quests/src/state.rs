//! Derived state types – pure projections from persisted data.

use uuid::Uuid;

use types::{QuestStatus, Student, StudentQuest};

/// Aggregated progress view for one student (read-only projection).
#[derive(Debug, Clone)]
pub struct StudentProgress {
    pub student_id:  Uuid,
    pub completed:   i64,
    pub failed:      i64,
    pub in_progress: i64,
    pub total_xp:    i32,
    pub level:       i32,
}

impl StudentProgress {
    /// Build the projection from the student record and their quest history.
    pub fn from_data(student: &Student, quests: &[StudentQuest]) -> Self {
        let completed   = quests.iter().filter(|q| q.status == QuestStatus::Completed).count() as i64;
        let failed      = quests.iter().filter(|q| q.status == QuestStatus::Failed).count() as i64;
        let in_progress = quests.iter().filter(|q| q.status == QuestStatus::InProgress).count() as i64;
        Self {
            student_id: student.id,
            completed,
            failed,
            in_progress,
            total_xp: student.xp,
            level:    student.level,
        }
    }
}
