//! `CORS_ORIGINS`: browser JavaScript on listed origins may call `/api`
//! with a bearer token; nobody else gets CORS headers.

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Request, StatusCode, header};
use loom::app::build_router;
use loom::state::AppState;
use sqlx::PgPool;
use tower::ServiceExt;

const ALLOWED: &str = "https://app.example.com";

fn app(pool: PgPool, origins: &[&str]) -> axum::Router {
    let origins: Vec<String> = origins.iter().map(|origin| origin.to_string()).collect();
    build_router(AppState::new(pool, "test-only-secret").with_cors_origins(&origins))
}

async fn preflight(app: &axum::Router, origin: &str) -> (StatusCode, HeaderMap) {
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/api/nodes")
        .header(header::ORIGIN, origin)
        .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
        .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization")
        .extension(ConnectInfo(std::net::SocketAddr::from(([127, 0, 0, 1], 1))))
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    (response.status(), response.headers().clone())
}

async fn get_with_origin(app: &axum::Router, origin: &str) -> HeaderMap {
    let request = Request::builder()
        .uri("/api/health")
        .header(header::ORIGIN, origin)
        .extension(ConnectInfo(std::net::SocketAddr::from(([127, 0, 0, 1], 1))))
        .body(Body::empty())
        .unwrap();
    app.clone()
        .oneshot(request)
        .await
        .unwrap()
        .headers()
        .clone()
}

#[sqlx::test]
async fn preflight_from_an_allowed_origin_succeeds_without_auth(pool: PgPool) {
    let app = app(pool, &[ALLOWED, "http://localhost:3000"]);
    let (status, headers) = preflight(&app, ALLOWED).await;
    assert!(status.is_success(), "{status}");
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOWED);
    assert!(
        headers[header::ACCESS_CONTROL_ALLOW_HEADERS]
            .to_str()
            .unwrap()
            .contains("authorization")
    );
    // Cookies never ride along cross-origin.
    assert!(
        !headers.contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS),
        "{headers:?}"
    );

    let (_, headers) = preflight(&app, "http://localhost:3000").await;
    assert_eq!(
        headers[header::ACCESS_CONTROL_ALLOW_ORIGIN],
        "http://localhost:3000"
    );
}

#[sqlx::test]
async fn other_origins_get_no_cors_headers(pool: PgPool) {
    let app = app(pool, &[ALLOWED]);
    let (_, headers) = preflight(&app, "https://evil.example.com").await;
    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    let headers = get_with_origin(&app, "https://evil.example.com").await;
    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));

    let headers = get_with_origin(&app, ALLOWED).await;
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOWED);
}

#[sqlx::test]
async fn trailing_slash_in_config_still_matches(pool: PgPool) {
    let app = app(pool, &["https://app.example.com/"]);
    let headers = get_with_origin(&app, ALLOWED).await;
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], ALLOWED);
}

#[sqlx::test]
async fn cors_is_off_unless_configured(pool: PgPool) {
    let app = app(pool, &[]);
    let (_, headers) = preflight(&app, ALLOWED).await;
    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
    let headers = get_with_origin(&app, ALLOWED).await;
    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));
}
