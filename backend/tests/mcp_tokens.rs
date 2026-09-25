use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use loom::app::build_router;
use loom::state::AppState;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test-only-secret";

fn app(pool: PgPool) -> axum::Router {
    build_router(AppState::new(pool, TEST_JWT_SECRET))
}

async fn send(app: &axum::Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

const TEST_PEER: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    12345,
);

/// Every request needs `ConnectInfo` attached, same as
/// `into_make_service_with_connect_info` would in production - the
/// `/api/auth/*` rate limiter's `SmartIpKeyExtractor` falls back to it
/// when there's no forwarded-for header, which `oneshot` never provides.
fn req(method: &str, uri: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .extension(ConnectInfo(TEST_PEER));
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn signup(app: &axum::Router, email: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/auth/signup",
            json!({"email": email, "password": "password123"}),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["access_token"].as_str().unwrap().to_owned()
}

#[sqlx::test]
async fn create_list_delete_happy_path(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "tokens@example.com").await;

    let (status, created) = send(
        &app,
        req(
            "POST",
            "/api/mcp/tokens",
            json!({"name": " laptop "}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["token"]["name"], "laptop");
    assert!(created["raw_token"].as_str().unwrap().starts_with("loom_"));
    let token_id = created["token"]["id"].as_str().unwrap().to_owned();

    let (status, list) = send(
        &app,
        req("GET", "/api/mcp/tokens", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert!(list[0].get("raw_token").is_none());
    assert!(list[0]["last_used_at"].is_null());

    let uri = format!("/api/mcp/tokens/{token_id}");
    let (status, _) = send(&app, req("DELETE", &uri, Value::Null, Some(&token))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, list) = send(
        &app,
        req("GET", "/api/mcp/tokens", Value::Null, Some(&token)),
    )
    .await;
    assert!(list.as_array().unwrap().is_empty());
}

#[sqlx::test]
async fn rejects_blank_name(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "blank@example.com").await;
    let (status, _) = send(
        &app,
        req(
            "POST",
            "/api/mcp/tokens",
            json!({"name": "  "}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn requires_login(pool: PgPool) {
    let app = app(pool);
    let (status, _) = send(&app, req("GET", "/api/mcp/tokens", Value::Null, None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn cannot_see_or_delete_another_users_tokens(pool: PgPool) {
    let app = app(pool);
    let owner = signup(&app, "owner@example.com").await;
    let intruder = signup(&app, "intruder@example.com").await;

    let (_, created) = send(
        &app,
        req(
            "POST",
            "/api/mcp/tokens",
            json!({"name": "mine"}),
            Some(&owner),
        ),
    )
    .await;
    let token_id = created["token"]["id"].as_str().unwrap().to_owned();

    let (_, list) = send(
        &app,
        req("GET", "/api/mcp/tokens", Value::Null, Some(&intruder)),
    )
    .await;
    assert!(list.as_array().unwrap().is_empty());

    let uri = format!("/api/mcp/tokens/{token_id}");
    let (status, _) = send(&app, req("DELETE", &uri, Value::Null, Some(&intruder))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, list) = send(
        &app,
        req("GET", "/api/mcp/tokens", Value::Null, Some(&owner)),
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn info_reports_disabled_by_default(pool: PgPool) {
    let app = app(pool);
    let (status, info) = send(&app, req("GET", "/api/mcp/info", Value::Null, None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(info, json!({"enabled": false, "url": null}));
}

#[sqlx::test]
async fn info_reports_url_when_enabled(pool: PgPool) {
    let app =
        build_router(AppState::new(pool, TEST_JWT_SECRET).with_mcp("https://loom.example.com/"));
    let (_, info) = send(&app, req("GET", "/api/mcp/info", Value::Null, None)).await;
    assert_eq!(
        info,
        json!({"enabled": true, "url": "https://loom.example.com/mcp"})
    );
}
