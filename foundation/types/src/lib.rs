//! `types` – foundation layer, Single Source of Truth.
//!
//! Contains every primitive type, ID, error, and trait contract.
//! **No I/O, no network, no database calls.**

pub mod errors;
pub mod ids;
pub mod models;
pub mod traits;

// ── Flat re-exports ───────────────────────────────────────────────────────────
pub use errors::{ForgeError, ForgeResult};
pub use ids::{
    AchievementId, AssessmentId, BiomeId, CertificationId,
    GroupId, QuestId, SandboxRunId, StudentId,
};
pub use models::achievement::{Achievement, AchievementType, Certification, CertifyRequest};
pub use models::assessment::{Assessment, AssessmentType, PerformanceMetrics, TestResult};
pub use models::biome::{Biome, BiomeDomain, BiomeState, BiomeSummary, ExploreRequest};
pub use models::group::{
    CreateGroupRequest, GroupMember, GroupMemberDetail, GroupProgressResponse,
    GroupStatus, StudyGroup,
};
pub use models::knowledge::{KnowledgeGraph, KnowledgeNode};
pub use models::quest::{
    CompleteQuestRequest, GenerateQuestRequest, Quest, QuestStatus, QuestType,
    StudentQuest, TestCase,
};
pub use models::sandbox::{
    PracticeRequest, ProgrammingLanguage, SandboxRun, SandboxStatus, SecurityScan,
    SubmitSolutionRequest,
};
pub use models::student::{
    EnrollRequest, EnrollResponse, Student, UpdateGoalsRequest, level_for_xp,
};
pub use traits::{AgentStrategy, Reducer};
