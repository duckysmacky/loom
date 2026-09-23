use axum::body::Body;
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

fn req(method: &str, uri: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");
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

#[sqlx::test]
async fn canvas_returns_callers_nodes_and_edges(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "canvas@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;
    let (status, _) = send(
        &app,
        req(
            "POST",
            "/api/edges",
            json!({"from_node_id": a, "to_node_id": b, "kind": "related"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, canvas) = send(
        &app,
        req("GET", "/api/board/canvas", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(canvas["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(canvas["edges"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn canvas_is_isolated_per_user(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "canvasowner@example.com").await;
    let token_b = signup(&app, "canvasintruder@example.com").await;
    create_node(&app, &token_a, "mine").await;

    let (_, canvas_b) = send(
        &app,
        req("GET", "/api/board/canvas", Value::Null, Some(&token_b)),
    )
    .await;
    assert_eq!(canvas_b["nodes"].as_array().unwrap().len(), 0);
    assert_eq!(canvas_b["edges"].as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn missing_or_garbage_token_returns_401(pool: PgPool) {
    let app = app(pool);

    let (missing_status, _) = send(&app, req("GET", "/api/board/canvas", Value::Null, None)).await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

    let (garbage_status, _) = send(
        &app,
        req(
            "GET",
            "/api/board/canvas",
            Value::Null,
            Some("not-a-real-token"),
        ),
    )
    .await;
    assert_eq!(garbage_status, StatusCode::UNAUTHORIZED);
}
