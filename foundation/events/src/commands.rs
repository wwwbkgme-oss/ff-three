//! Academy commands – intents that trigger state changes via events.
//!
//! Commands are validated; invalid commands return errors without emitting events.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use types::ProgrammingLanguage;

/// Every user or system intention is expressed as one of these variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcademyCommand {
    // ── Student ───────────────────────────────────────────────────────────────
    EnrollStudent {
        username: String,
        email: String,
        topic: Option<String>,
        goals: Vec<String>,
    },
    UpdateGoals {
        student_id: Uuid,
        goals: Vec<String>,
    },

    // ── Quests ────────────────────────────────────────────────────────────────
    GenerateQuest {
        goal: String,
        biome_id: Option<Uuid>,
        difficulty: Option<i32>,
    },
    StartQuest {
        student_id: Uuid,
        quest_id: Uuid,
    },
    SubmitSolution {
        student_id: Uuid,
        quest_id: Uuid,
        code: String,
        language: ProgrammingLanguage,
    },

    // ── World ─────────────────────────────────────────────────────────────────
    EnterBiome {
        student_id: Uuid,
        biome_slug: String,
    },
    RecalculateBiomeState {
        biome_id: Uuid,
    },

    // ── Collaboration ─────────────────────────────────────────────────────────
    CreateGroup {
        name: String,
        goal: String,
        biome_id: Option<Uuid>,
        max_members: Option<i32>,
    },
    JoinGroup {
        group_id: Uuid,
        student_id: Uuid,
    },

    // ── Achievements ──────────────────────────────────────────────────────────
    RequestCertification {
        student_id: Uuid,
        path: String,
    },
}
