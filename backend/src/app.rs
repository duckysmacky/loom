use std::time::Duration;

use axum::{
    Router,
    routing::{delete, get, post},
};
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::trace::TraceLayer;

use crate::handlers;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    // Peer IP is the only option pre-auth (no account to key on yet), and
    // there's no reverse proxy in front of this app to trust a forwarded-for
    // header from - PeerIpKeyExtractor is tower_governor's default.
    let login_governor_config = GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(5)
        .finish()
        .expect("valid governor config");

    // governor's keyed rate limiter store grows unboundedly without this.
    let login_limiter = login_governor_config.limiter().clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(300));
            login_limiter.retain_recent();
        }
    });

    let auth_routes = Router::new()
        .route("/signup", post(handlers::auth::signup))
        .route(
            "/login",
            post(handlers::auth::login).route_layer(GovernorLayer::new(login_governor_config)),
        )
        .route("/refresh", post(handlers::auth::refresh))
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

    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/health", get(health))
                .nest("/auth", auth_routes)
                .nest("/nodes", node_routes)
                .nest("/topics", topic_routes),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> &'static str {
    "ok"
}
