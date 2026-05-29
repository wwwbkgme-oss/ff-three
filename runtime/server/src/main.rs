//! ForgeFabrik Academy — HTTP server entry point.
//!
//! Boot-Sequenz:
//! 1. Konfiguration aus ENV laden
//! 2. PostgreSQL + Redis verbinden
//! 3. Migrations ausführen
//! 4. AppState initialisieren (Orchestrator + LLM-Driver)
//! 5. Simulation-Worker spawnen (P2.2 — Tick-Loop)
//! 6. Projection-Worker spawnen (P2.3 — Read-Model catch-up)
//! 7. Axum-HTTP-Server starten

use uuid::Uuid;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod event_store;
mod llm;
mod routes;
mod sim;
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

    // ── Migrations ────────────────────────────────────────────────────────────
    db::run_migrations(&state.db).await?;
    tracing::info!("Migrations: OK");

    // ── Simulation-Worker (P2.2) ──────────────────────────────────────────────
    // Realm-ID: deterministisch aus Config-Hash ableiten
    let realm = Uuid::new_v5(&Uuid::NAMESPACE_URL, config.host.as_bytes());
    let (tick_handle, tick_counter) = sim::tick_worker::spawn(state.clone(), realm);
    tracing::info!(?realm, "tick_worker: gestartet");

    // ── Projection-Worker (P2.3) ──────────────────────────────────────────────
    let proj_handle = sim::projection_worker::spawn(state.clone());
    tracing::info!("projection_worker: gestartet");

    // ── HTTP-Server ───────────────────────────────────────────────────────────
    let app  = routes::router(state);
    let addr: std::net::SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Listening on {addr}");

    // Graceful shutdown: HTTP-Server fertig → Worker beenden
    tokio::select! {
        result = axum::serve(listener, app) => {
            result?;
        }
        _ = tick_handle => {
            tracing::error!("tick_worker beendet unerwartet");
        }
        _ = proj_handle => {
            tracing::error!("projection_worker beendet unerwartet");
        }
    }

    Ok(())
}
