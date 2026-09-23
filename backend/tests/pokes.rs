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

async fn poke(app: &axum::Router, token: &str, node_id: &str) -> (StatusCode, Value) {
    send(
        app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/pokes"),
            Value::Null,
            Some(token),
        ),
    )
    .await
}

async fn get_node(app: &axum::Router, token: &str, id: &str) -> Value {
    send(
        app,
        req("GET", &format!("/api/nodes/{id}"), Value::Null, Some(token)),
    )
    .await
    .1
}

#[sqlx::test]
async fn create_poke_happy_path(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pokecreate@example.com").await;
    let node_id = create_node(&app, &token, "N").await;

    let (status, body) = poke(&app, &token, &node_id).await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["id"].as_str().is_some());
    assert!(body["poked_at"].as_str().is_some());
}

#[sqlx::test]
async fn list_pokes_ordered_newest_first(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pokelist@example.com").await;
    let node_id = create_node(&app, &token, "N").await;

    let (_, first) = poke(&app, &token, &node_id).await;
    let (_, second) = poke(&app, &token, &node_id).await;

    let (status, list) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/pokes"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list = list.as_array().unwrap();
    assert_eq!(list.len(), 2);
    // Newest first: the second poke's id leads.
    assert_eq!(list[0]["id"], second["id"]);
    assert_eq!(list[1]["id"], first["id"]);
}

#[sqlx::test]
async fn last_poked_at_reflects_max_and_null_when_never_poked(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "lastpoked@example.com").await;
    let node_id = create_node(&app, &token, "N").await;

    let never = get_node(&app, &token, &node_id).await;
    assert_eq!(never["last_poked_at"], Value::Null);

    poke(&app, &token, &node_id).await;
    let (_, latest) = poke(&app, &token, &node_id).await;

    let after = get_node(&app, &token, &node_id).await;
    assert_eq!(after["last_poked_at"], latest["poked_at"]);
}

#[sqlx::test]
async fn ownership_scoping_returns_404_for_another_users_node(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "pokeowner@example.com").await;
    let token_b = signup(&app, "pokeintruder@example.com").await;
    let node_id = create_node(&app, &token_a, "mine").await;

    let (create_status, _) = poke(&app, &token_b, &node_id).await;
    assert_eq!(create_status, StatusCode::NOT_FOUND);

    let (list_status, _) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/pokes"),
            Value::Null,
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(list_status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn list_nodes_agrees_with_get_node_on_last_poked_at(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pokeconsistency@example.com").await;
    let node_id = create_node(&app, &token, "N").await;
    poke(&app, &token, &node_id).await;

    let via_get = get_node(&app, &token, &node_id).await;
    let (_, list) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token))).await;
    let via_list = list
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == node_id)
        .unwrap();

    assert_eq!(via_get["last_poked_at"], via_list["last_poked_at"]);
}

#[sqlx::test]
async fn deleting_node_cascades_pokes(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pokecascade@example.com").await;
    let node_id = create_node(&app, &token, "N").await;
    poke(&app, &token, &node_id).await;

    let (delete_status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    // Node is gone, so poking/listing it now 404s regardless of orphaned
    // rows - the DB's ON DELETE CASCADE means there shouldn't be any, but
    // this confirms the API surface behaves correctly either way.
    let (poke_status, _) = poke(&app, &token, &node_id).await;
    assert_eq!(poke_status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn missing_or_garbage_token_returns_401(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "poke401owner@example.com").await;
    let node_id = create_node(&app, &token, "N").await;

    let (missing_status, _) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/pokes"),
            Value::Null,
            None,
        ),
    )
    .await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

    let (garbage_status, _) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/pokes"),
            Value::Null,
            Some("not-a-real-token"),
        ),
    )
    .await;
    assert_eq!(garbage_status, StatusCode::UNAUTHORIZED);
}
