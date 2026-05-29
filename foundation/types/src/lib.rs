//! `types` – foundation layer, Single Source of Truth.
//!
//! Contains every primitive type, ID, error, trait contract, and simulation
//! time primitive.  **No I/O, no network, no database calls.**

pub mod errors;
pub mod ids;
pub mod models;
pub mod snapshot;
pub mod time;
pub mod traits;

// ── Flat re-exports ───────────────────────────────────────────────────────────
pub use errors::{ForgeError, ForgeResult};
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
pub use traits::{AggregateRoot, AgentStrategy, CommandContext, CommandHandler, Reducer};
pub use snapshot::{DeterministicHash, WorldSnapshot};
pub use time::{WorldTick, DAY_LENGTH_TICKS, day_fraction_to_tick, tick_of_day};
pub use ids::{
    // existing
    AchievementId, AssessmentId, BiomeId, CertificationId,
    GroupId, QuestId, SandboxRunId, StudentId,
    // character / simulation domain
    ActorId, CharacterId, CorrelationId, EpisodeId,
    EventId, FactionId, GoalId, LocationId, RealmId, SnapshotId,
};
