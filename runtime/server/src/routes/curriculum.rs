use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use crate::{error::ServerResult, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new().route("/curriculum", get(list_paths))
}

const PATHS: &[(&str, &str, &[&str])] = &[
    ("full-stack",          "Full-Stack Developer",     &["web","languages","systems"]),
    ("systems-programming", "Systems Programmer",       &["systems","algorithms","languages"]),
    ("machine-learning",    "Machine Learning Engineer",&["artificial-intelligence","mathematics","data-science"]),
    ("security-specialist", "Security Specialist",      &["security","systems","algorithms"]),
    ("data-engineering",    "Data Engineer",            &["data-science","mathematics","systems"]),
    ("algorithm-expert",    "Algorithm Expert",         &["algorithms","mathematics"]),
];

async fn list_paths() -> ServerResult<Json<Value>> {
    let paths: Vec<Value> = PATHS.iter().map(|(id, title, biomes)| json!({
        "id": id, "title": title, "required_biomes": biomes,
        "min_quests": 5, "certification": format!("forge:cert/{id}"),
    })).collect();
    let count = paths.len();
    Ok(Json(json!({ "paths": paths, "count": count })))
}
