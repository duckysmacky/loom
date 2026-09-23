use std::net::SocketAddr;

use anyhow::{Context, Result};
use loom::{app, config::Config, state::AppState};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use tokio::signal;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;

    let connect_options = PgConnectOptions::new()
        .host(&config.db_host)
        .port(config.db_port)
        .username(&config.db_user)
        .password(&config.db_password)
        .database(&config.db_name);

    let pool = connect_with_retries(connect_options).await?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    let state = AppState::new(pool, &config.jwt_secret);
    let app = app::build_router(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("failed to bind {}", config.bind_addr))?;

    tracing::info!("listening on {}", config.bind_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("server error")?;

    Ok(())
}

/// The DB container/DNS entry may not be ready the instant this one starts
/// (e.g. a fresh `docker compose up`) - retry briefly instead of failing
/// fast on what's usually a few seconds' race, not a real outage.
async fn connect_with_retries(options: PgConnectOptions) -> Result<sqlx::PgPool> {
    const MAX_ATTEMPTS: u32 = 10;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(1);

    for attempt in 1..=MAX_ATTEMPTS {
        match PgPoolOptions::new().connect_with(options.clone()).await {
            Ok(pool) => return Ok(pool),
            Err(error) if attempt < MAX_ATTEMPTS => {
                tracing::warn!(attempt, %error, "database not ready yet, retrying");
                tokio::time::sleep(DELAY).await;
            }
            Err(error) => return Err(error).context("failed to connect to database"),
        }
    }

    unreachable!("loop always returns on its final attempt")
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutting down");
}
