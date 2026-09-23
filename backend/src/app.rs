use std::time::Duration;

use axum::response::IntoResponse;
use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_governor::{
    GovernorLayer, errors::GovernorError, governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
};
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::handlers::error::ApiError;
use crate::state::AppState;

/// Governor's default 429/500 responses are plain text - map them to the
/// same `{"error": ...}` shape every other rejection in this API uses.
fn governor_error_response(error: GovernorError) -> axum::response::Response {
    match error {
        GovernorError::TooManyRequests { .. } => ApiError::TooManyRequests.into_response(),
        _ => ApiError::Internal(anyhow::anyhow!("rate limiter error: {error}")).into_response(),
    }
}

pub fn build_router(state: AppState) -> Router {
    // SmartIpKeyExtractor reads X-Forwarded-For/X-Real-IP/Forwarded (falling
    // back to peer IP) - PeerIpKeyExtractor (tower_governor's default) would
    // key every request behind the nginx reverse proxy PROJECT_OVERVIEW
    // describes on the same peer IP (the proxy's), turning this into one
    // shared bucket for every client. Only safe because the app's port is
    // bound to 127.0.0.1 in docker-compose.yml - nothing but that trusted
    // proxy can set these headers directly.
    let auth_governor_config = GovernorConfigBuilder::default()
        .key_extractor(SmartIpKeyExtractor)
        .per_second(10)
        .burst_size(5)
        .finish()
        .expect("valid governor config");

    // governor's keyed rate limiter store grows unboundedly without this.
    let auth_limiter = auth_governor_config.limiter().clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(300));
            auth_limiter.retain_recent();
        }
    });

    // Signup, login, refresh and password change all either run Argon2 or
    // write a DB row - all get the same per-IP throttle so none is a free
    // CPU/DB sink.
    // logout/me are cheap, authenticated-only reads/writes and don't need it.
    let throttled_auth_routes = Router::new()
        .route("/signup", post(handlers::auth::signup))
        .route("/login", post(handlers::auth::login))
        .route("/refresh", post(handlers::auth::refresh))
        .route("/password", post(handlers::auth::change_password))
        .route_layer(
            GovernorLayer::new(auth_governor_config).error_handler(governor_error_response),
        );

    let auth_routes = throttled_auth_routes
        .route("/logout", post(handlers::auth::logout))
        .route("/me", get(handlers::auth::me));

    let node_routes = Router::new()
        .route(
            "/",
            get(handlers::nodes::list).post(handlers::nodes::create),
        )
        .route(
            "/{id}",
            get(handlers::nodes::get)
                .patch(handlers::nodes::update)
                .delete(handlers::nodes::delete),
        )
        .route("/{id}/topics", post(handlers::nodes::attach_topic))
        .route(
            "/{id}/topics/{topic_id}",
            delete(handlers::nodes::detach_topic),
        )
        .route(
            "/{id}/pokes",
            get(handlers::pokes::list).post(handlers::pokes::create),
        );

    let topic_routes = Router::new()
        .route(
            "/",
            get(handlers::topics::list).post(handlers::topics::create),
        )
        .route(
            "/{id}",
            get(handlers::topics::get)
                .patch(handlers::topics::update)
                .delete(handlers::topics::delete),
        );

    let edge_routes = Router::new()
        .route(
            "/",
            get(handlers::edges::list).post(handlers::edges::create),
        )
        .route("/{id}", delete(handlers::edges::delete));

    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/health", get(health))
                .route("/board/canvas", get(handlers::board::canvas))
                .route("/dashboard", get(handlers::dashboard::get))
                .nest("/auth", auth_routes)
                .nest("/nodes", node_routes)
                .nest("/topics", topic_routes)
                .nest("/edges", edge_routes),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> &'static str {
    "ok"
}
