//! Personal access tokens on the REST API: which routes take them, what a
//! `read` token may do, and what must never accept one.

use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use loom::app::build_router;
use loom::auth::refresh_token::hash_token;
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

/// Creates a personal token for the web session `jwt`; returns (id, raw).
async fn create_token(app: &axum::Router, jwt: &str, scope: &str) -> (String, String) {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/tokens",
            json!({"name": "t", "scope": scope}),
            Some(jwt),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    (
        body["token"]["id"].as_str().unwrap().to_owned(),
        body["raw_token"].as_str().unwrap().to_owned(),
    )
}

async fn create_node(app: &axum::Router, token: &str, title: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "project", "title": title}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

#[sqlx::test]
async fn token_reads_the_data_routes(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "reader@example.com").await;
    create_node(&app, &jwt, "mine").await;
    let (_, token) = create_token(&app, &jwt, "read").await;

    for uri in [
        "/api/nodes",
        "/api/nodes?status=queued&focus=secondary",
        "/api/dashboard",
        "/api/board/canvas",
        "/api/topics",
        "/api/edges",
    ] {
        let (status, _) = send(&app, req("GET", uri, Value::Null, Some(&token))).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
    }
    let (_, nodes) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token))).await;
    assert_eq!(nodes.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn read_token_cannot_write_but_read_write_can(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "writer@example.com").await;
    let node_id = create_node(&app, &jwt, "mine").await;
    let (_, read) = create_token(&app, &jwt, "read").await;
    let (_, read_write) = create_token(&app, &jwt, "read_write").await;

    let node_uri = format!("/api/nodes/{node_id}");
    for (method, uri, body) in [
        ("POST", "/api/nodes", json!({"kind": "idea", "title": "x"})),
        ("PATCH", node_uri.as_str(), json!({"title": "renamed"})),
        ("DELETE", node_uri.as_str(), Value::Null),
    ] {
        let (status, _) = send(&app, req(method, uri, body, Some(&read))).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {uri}");
    }
    // Nothing changed.
    let (_, node) = send(&app, req("GET", &node_uri, Value::Null, Some(&read))).await;
    assert_eq!(node["title"], "mine");

    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &node_uri,
            json!({"title": "renamed"}),
            Some(&read_write),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(
        &app,
        req("DELETE", &node_uri, Value::Null, Some(&read_write)),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn token_is_rejected_on_account_routes(pool: PgPool) {
    let app = build_router(AppState::new(pool, TEST_JWT_SECRET).with_mcp("https://loom.test"));
    let jwt = signup(&app, "account@example.com").await;
    let (token_id, token) = create_token(&app, &jwt, "read_write").await;

    for (method, uri, body) in [
        ("GET", "/api/auth/me".to_owned(), Value::Null),
        (
            "POST",
            "/api/auth/password".to_owned(),
            json!({"current_password": "password123", "new_password": "hunter2hunter2"}),
        ),
        ("GET", "/api/tokens".to_owned(), Value::Null),
        ("POST", "/api/tokens".to_owned(), json!({"name": "mint"})),
        ("DELETE", format!("/api/tokens/{token_id}"), Value::Null),
        ("GET", "/api/oauth/clients".to_owned(), Value::Null),
    ] {
        let (status, _) = send(&app, req(method, &uri, body, Some(&token))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method} {uri}");
    }

    // The token survived all of that, and the JWT still works there.
    let (status, list) = send(&app, req("GET", "/api/tokens", Value::Null, Some(&jwt))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn oauth_tokens_do_not_work_on_the_api(pool: PgPool) {
    let app = app(pool.clone());
    let jwt = signup(&app, "oauth@example.com").await;
    let user_id: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO oauth_clients (client_id, client_name, redirect_uris) VALUES ('c', 'c', '{}')",
    )
    .execute(&pool)
    .await
    .unwrap();
    // Same prefix as a personal token, so only the stored kind can keep it out.
    let raw = "loom_oauth-access-token";
    sqlx::query(
        "INSERT INTO access_tokens (user_id, kind, client_id, token_hash, expires_at)
         VALUES ($1, 'access', 'c', $2, now() + interval '1 hour')",
    )
    .bind(user_id)
    .bind(hash_token(raw))
    .execute(&pool)
    .await
    .unwrap();

    let (status, _) = send(&app, req("GET", "/api/nodes", Value::Null, Some(raw))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // ...and the failed attempt didn't count as use.
    let used: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT last_used_at FROM access_tokens WHERE kind = 'access'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(used.is_none());
    // The user's own session is unaffected.
    let (status, _) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&jwt))).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn revoked_expired_and_bogus_tokens_are_rejected(pool: PgPool) {
    let app = app(pool.clone());
    let jwt = signup(&app, "dead@example.com").await;
    let (revoked_id, revoked) = create_token(&app, &jwt, "read").await;
    let (_, expired) = create_token(&app, &jwt, "read").await;
    send(
        &app,
        req(
            "DELETE",
            &format!("/api/tokens/{revoked_id}"),
            Value::Null,
            Some(&jwt),
        ),
    )
    .await;
    sqlx::query(
        "UPDATE access_tokens SET expires_at = now() - interval '1 minute' WHERE token_hash = $1",
    )
    .bind(hash_token(&expired))
    .execute(&pool)
    .await
    .unwrap();

    for token in [revoked.as_str(), expired.as_str(), "loom_not-a-real-token"] {
        let (status, _) = send(&app, req("GET", "/api/nodes", Value::Null, Some(token))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{token}");
    }
    let (status, _) = send(&app, req("GET", "/api/nodes", Value::Null, None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn token_only_sees_its_own_users_data(pool: PgPool) {
    let app = app(pool);
    let alice = signup(&app, "alice@example.com").await;
    let bob = signup(&app, "bob@example.com").await;
    let alices_node = create_node(&app, &alice, "alice's").await;
    let (_, bob_token) = create_token(&app, &bob, "read_write").await;

    let (_, nodes) = send(
        &app,
        req("GET", "/api/nodes", Value::Null, Some(&bob_token)),
    )
    .await;
    assert!(nodes.as_array().unwrap().is_empty());

    let uri = format!("/api/nodes/{alices_node}");
    for (method, body) in [
        ("GET", Value::Null),
        ("PATCH", json!({"title": "hijacked"})),
        ("DELETE", Value::Null),
    ] {
        let (status, _) = send(&app, req(method, &uri, body, Some(&bob_token))).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{method}");
    }
    let (_, node) = send(&app, req("GET", &uri, Value::Null, Some(&alice))).await;
    assert_eq!(node["title"], "alice's");
}

#[sqlx::test]
async fn web_session_jwt_still_works_on_data_routes(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "session@example.com").await;
    create_node(&app, &jwt, "mine").await;
    let (status, _) = send(&app, req("GET", "/api/dashboard", Value::Null, Some(&jwt))).await;
    assert_eq!(status, StatusCode::OK);
}

#[sqlx::test]
async fn token_use_is_recorded(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "used@example.com").await;
    let (_, token) = create_token(&app, &jwt, "read").await;
    send(&app, req("GET", "/api/nodes", Value::Null, Some(&token))).await;
    let (_, list) = send(&app, req("GET", "/api/tokens", Value::Null, Some(&jwt))).await;
    assert!(list[0]["last_used_at"].is_string());
}
