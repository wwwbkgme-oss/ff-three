use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "biome_domain", rename_all = "snake_case")]
pub enum BiomeDomain {
    Algorithms, Security, ArtificialIntelligence,
    Systems, Languages, Web, DataScience, Mathematics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "biome_state", rename_all = "snake_case")]
pub enum BiomeState { Enlightened, Clouded, Confused, Mastered }

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Biome {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub domain: BiomeDomain,
    pub description: String,
    pub lore: String,
    pub min_difficulty: i32,
    pub max_difficulty: i32,
    pub unlock_requirements: Vec<String>,
    pub state: BiomeState,
    pub active_students: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSummary {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub domain: BiomeDomain,
    pub state: BiomeState,
    pub active_students: i32,
    pub available_quests: i32,
}

#[derive(Debug, Deserialize)]
pub struct ExploreRequest {
    pub student_id: Uuid,
    pub biome_slug: String,
}
