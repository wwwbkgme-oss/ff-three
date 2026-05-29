//! Type-safe ID newtypes – one per aggregate root.
//! Using newtypes prevents accidentally passing a QuestId where a StudentId is expected.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
        #[sqlx(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            pub fn new() -> Self { Self(Uuid::new_v4()) }
            pub fn inner(&self) -> Uuid { self.0 }
        }

        impl Default for $name {
            fn default() -> Self { Self::new() }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self { Self(u) }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Uuid { id.0 }
        }
    };
}

define_id!(StudentId);
define_id!(QuestId);
define_id!(BiomeId);
define_id!(GroupId);
define_id!(AssessmentId);
define_id!(SandboxRunId);
define_id!(AchievementId);
define_id!(CertificationId);
