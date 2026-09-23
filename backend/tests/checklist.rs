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
            json!({"kind": "project", "title": title}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

async fn add_item(app: &axum::Router, token: &str, node_id: &str, title: &str) -> Value {
    let (status, body) = send(
        app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/checklist"),
            json!({"title": title}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body
}

async fn get_node(app: &axum::Router, token: &str, node_id: &str) -> Value {
    let (status, body) = send(
        app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body
}

#[sqlx::test]
async fn items_append_in_order_and_list_back(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "list@example.com").await;
    let node_id = create_node(&app, &token, "project").await;

    let first = add_item(&app, &token, &node_id, "  write schema  ").await;
    let second = add_item(&app, &token, &node_id, "wire routes").await;
    assert_eq!(first["title"], "write schema");
    assert_eq!(first["done"], false);
    assert_eq!(first["position"], 0);
    assert_eq!(second["position"], 1);

    let (status, items) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/checklist"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let titles: Vec<&str> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, vec!["write schema", "wire routes"]);
}

#[sqlx::test]
async fn checklist_drives_node_progress(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "progress@example.com").await;
    let node_id = create_node(&app, &token, "project").await;
    assert_eq!(
        get_node(&app, &token, &node_id).await["checklist_progress"],
        Value::Null
    );

    let item = add_item(&app, &token, &node_id, "one").await;
    add_item(&app, &token, &node_id, "two").await;
    let item_id = item["id"].as_str().unwrap();

    let (status, updated) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/checklist/{item_id}"),
            json!({"done": true}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["done"], true);
    assert_eq!(updated["title"], "one");

    assert_eq!(
        get_node(&app, &token, &node_id).await["checklist_progress"],
        json!({"done": 1, "total": 2})
    );
    // The PATCH-returned node carries it too (a different query path).
    let (_, patched) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"title": "renamed"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(
        patched["checklist_progress"],
        json!({"done": 1, "total": 2})
    );

    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/checklist/{item_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        get_node(&app, &token, &node_id).await["checklist_progress"],
        json!({"done": 0, "total": 1})
    );
}

#[sqlx::test]
async fn blank_titles_are_rejected(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "blank@example.com").await;
    let node_id = create_node(&app, &token, "project").await;

    let (status, _) = send(
        &app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/checklist"),
            json!({"title": "   "}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let item = add_item(&app, &token, &node_id, "ok").await;
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/checklist/{}", item["id"].as_str().unwrap()),
            json!({"title": ""}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn another_users_checklist_is_invisible(pool: PgPool) {
    let app = app(pool);
    let owner = signup(&app, "owner@example.com").await;
    let intruder = signup(&app, "intruder@example.com").await;
    let node_id = create_node(&app, &owner, "private").await;
    let item = add_item(&app, &owner, &node_id, "secret").await;
    let item_uri = format!("/api/checklist/{}", item["id"].as_str().unwrap());
    let list_uri = format!("/api/nodes/{node_id}/checklist");

    let (list_status, _) = send(&app, req("GET", &list_uri, Value::Null, Some(&intruder))).await;
    assert_eq!(list_status, StatusCode::NOT_FOUND);
    let (add_status, _) = send(
        &app,
        req("POST", &list_uri, json!({"title": "x"}), Some(&intruder)),
    )
    .await;
    assert_eq!(add_status, StatusCode::NOT_FOUND);
    let (patch_status, _) = send(
        &app,
        req("PATCH", &item_uri, json!({"done": true}), Some(&intruder)),
    )
    .await;
    assert_eq!(patch_status, StatusCode::NOT_FOUND);
    let (delete_status, _) =
        send(&app, req("DELETE", &item_uri, Value::Null, Some(&intruder))).await;
    assert_eq!(delete_status, StatusCode::NOT_FOUND);

    // Untouched for the owner.
    let (_, items) = send(&app, req("GET", &list_uri, Value::Null, Some(&owner))).await;
    assert_eq!(items[0]["done"], false);
}
