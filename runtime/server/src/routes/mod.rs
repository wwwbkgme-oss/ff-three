//! Route stubs – wired fully in TODO 4 (HTTP routes).

use axum::Router;
use crate::state::AppState;

pub fn router(_state: AppState) -> Router {
    Router::new()
}
