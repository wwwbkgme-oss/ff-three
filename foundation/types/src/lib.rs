//! `types` – foundation layer, Single Source of Truth.
//!
//! Contains every primitive type, ID, error, trait contract, and simulation
//! time primitive.  **No I/O, no network, no database calls.**

pub mod errors;
pub mod ids;
pub mod models;
pub mod rng;
pub mod snapshot;
pub mod tick_context;
pub mod time;
pub mod traits;

// ── Flat re-exports ───────────────────────────────────────────────────────────
pub use errors::{ForgeError, ForgeResult};
pub use ids::{
    // existing
    AchievementId,
    // character / simulation domain
    ActorId,
    AssessmentId,
    BiomeId,
    CertificationId,
    CharacterId,
    CorrelationId,
    EpisodeId,
    EventId,
    FactionId,
    GoalId,
    GroupId,
    LocationId,
    QuestId,
    RealmId,
    SandboxRunId,
    SnapshotId,
    StudentId,
};
pub use models::achievement::{Achievement, AchievementType, Certification, CertifyRequest};
pub use models::assessment::{Assessment, AssessmentType, PerformanceMetrics, TestResult};
pub use models::biome::{Biome, BiomeDomain, BiomeState, BiomeSummary, ExploreRequest};
pub use models::group::{
    CreateGroupRequest, GroupMember, GroupMemberDetail, GroupProgressResponse, GroupStatus,
    StudyGroup,
};
pub use models::knowledge::{KnowledgeGraph, KnowledgeNode};
pub use models::quest::{
    CompleteQuestRequest, GenerateQuestRequest, Quest, QuestStatus, QuestType, StudentQuest,
    TestCase,
};
pub use models::sandbox::{
    PracticeRequest, ProgrammingLanguage, SandboxRun, SandboxStatus, SecurityScan,
    SubmitSolutionRequest,
};
pub use models::student::{
    level_for_xp, EnrollRequest, EnrollResponse, Student, UpdateGoalsRequest,
};
pub use rng::DeterministicRng;
pub use snapshot::{DeterministicHash, WorldSnapshot};
pub use tick_context::TickContext;
pub use time::{day_fraction_to_tick, tick_of_day, WorldTick, DAY_LENGTH_TICKS};
pub use traits::{AgentStrategy, AggregateRoot, CommandContext, CommandHandler, Reducer};
