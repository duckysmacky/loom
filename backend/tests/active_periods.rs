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

async fn create_node(app: &axum::Router, token: &str, title: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "idea", "title": title}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

async fn add_period(app: &axum::Router, token: &str, node_id: &str, body: Value) -> Value {
    let (status, period) = send(
        app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/periods"),
            body,
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    period
}

#[sqlx::test]
async fn periods_are_created_listed_updated_and_deleted(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "periods@example.com").await;
    let node_id = create_node(&app, &token, "a node").await;

    let period = add_period(
        &app,
        &token,
        &node_id,
        json!({"started_at": "2026-09-01T00:00:00Z", "ended_at": "2026-09-05T00:00:00Z"}),
    )
    .await;
    assert_eq!(period["started_at"], "2026-09-01T00:00:00Z");
    assert_eq!(period["ended_at"], "2026-09-05T00:00:00Z");
    let period_id = period["id"].as_str().unwrap().to_owned();

    let (status, list) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/periods"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // A tri-state null on ended_at reopens it.
    let (status, updated) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/periods/{period_id}"),
            json!({"ended_at": null}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["ended_at"], Value::Null);
    assert_eq!(updated["started_at"], "2026-09-01T00:00:00Z"); // untouched field kept

    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/periods/{period_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, list) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/periods"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn create_rejects_ended_before_started(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "invalidrange@example.com").await;
    let node_id = create_node(&app, &token, "a node").await;

    let (status, _) = send(
        &app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/periods"),
            json!({"started_at": "2026-09-05T00:00:00Z", "ended_at": "2026-09-01T00:00:00Z"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn update_rejects_ended_before_started_via_db_check(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "updateinvalid@example.com").await;
    let node_id = create_node(&app, &token, "a node").await;
    let period = add_period(
        &app,
        &token,
        &node_id,
        json!({"started_at": "2026-09-05T00:00:00Z"}),
    )
    .await;
    let period_id = period["id"].as_str().unwrap();

    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/periods/{period_id}"),
            json!({"ended_at": "2026-09-01T00:00:00Z"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn another_users_periods_are_invisible(pool: PgPool) {
    let app = app(pool);
    let owner = signup(&app, "periodowner@example.com").await;
    let intruder = signup(&app, "periodintruder@example.com").await;
    let node_id = create_node(&app, &owner, "private").await;
    let period = add_period(
        &app,
        &owner,
        &node_id,
        json!({"started_at": "2026-09-01T00:00:00Z"}),
    )
    .await;
    let period_id = period["id"].as_str().unwrap();
    let list_uri = format!("/api/nodes/{node_id}/periods");
    let period_uri = format!("/api/periods/{period_id}");

    let (list_status, _) = send(&app, req("GET", &list_uri, Value::Null, Some(&intruder))).await;
    assert_eq!(list_status, StatusCode::NOT_FOUND);
    let (add_status, _) = send(
        &app,
        req(
            "POST",
            &list_uri,
            json!({"started_at": "2026-09-01T00:00:00Z"}),
            Some(&intruder),
        ),
    )
    .await;
    assert_eq!(add_status, StatusCode::NOT_FOUND);
    let (patch_status, _) = send(
        &app,
        req(
            "PATCH",
            &period_uri,
            json!({"ended_at": "2026-09-02T00:00:00Z"}),
            Some(&intruder),
        ),
    )
    .await;
    assert_eq!(patch_status, StatusCode::NOT_FOUND);
    let (delete_status, _) = send(
        &app,
        req("DELETE", &period_uri, Value::Null, Some(&intruder)),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NOT_FOUND);

    // Untouched for the owner.
    let (_, list) = send(&app, req("GET", &list_uri, Value::Null, Some(&owner))).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
}
