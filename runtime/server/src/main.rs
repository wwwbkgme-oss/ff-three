//! ForgeFabrik Academy – HTTP server entry point.
//!
//! Boots configuration → connection pools → migrations → Axum server.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod event_store;
mod llm;
mod routes;
mod state;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "server=debug,tower_http=info,sqlx=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;
    tracing::info!("ForgeFabrik Academy starting on {}:{}", config.host, config.port);

    let state = AppState::new(&config).await?;
    db::run_migrations(&state.db).await?;

    let app = routes::router(state);
    let addr: std::net::SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {addr}");
    axum::serve(listener, app).await?;

    Ok(())
}
