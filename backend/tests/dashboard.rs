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

async fn create_node(app: &axum::Router, token: &str, body: Value) -> String {
    let (status, body) = send(app, req("POST", "/api/nodes", body, Some(token))).await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

#[sqlx::test]
async fn counts_match_fixtures_exactly(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "counts@example.com").await;

    create_node(&app, &token, json!({"kind": "idea", "title": "a"})).await;
    create_node(&app, &token, json!({"kind": "idea", "title": "b"})).await;
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "c", "status": "active"}),
    )
    .await;
    create_node(
        &app,
        &token,
        json!({"kind": "study", "title": "d", "status": "active", "archived": true}),
    )
    .await;

    let (status, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let counts = &dashboard["counts"];
    assert_eq!(counts["total"], 4);
    assert_eq!(counts["by_status"]["idea"], 0);
    // The archived study keeps its status but is counted apart.
    assert_eq!(counts["by_status"]["active"], 1);
    assert_eq!(counts["by_status"]["archived"], 1);
    assert_eq!(counts["by_status"]["queued"], 0);
    assert_eq!(counts["by_kind"]["idea"], 2);
    assert_eq!(counts["by_kind"]["project"], 1);
    assert_eq!(counts["by_kind"]["study"], 1);
    assert_eq!(counts["by_kind"]["path"], 0);
    assert_eq!(counts["ideas"], 2);
    assert_eq!(counts["blocked"], 0);
}

async fn add_edge(app: &axum::Router, token: &str, from: &str, to: &str, kind: &str) {
    let (status, _) = send(
        app,
        req(
            "POST",
            "/api/edges",
            json!({"from_node_id": from, "to_node_id": to, "kind": kind}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
}

fn titles(list: &Value) -> Vec<String> {
    list.as_array()
        .unwrap()
        .iter()
        .map(|n| n["title"].as_str().unwrap().to_owned())
        .collect()
}

#[sqlx::test]
async fn active_list_ranks_by_tier_then_unblocked(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "activelist@example.com").await;

    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "queued", "status": "queued", "focus": "primary"}),
    )
    .await;
    let blocked = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "blocked", "status": "active", "focus": "primary"}),
    )
    .await;
    let blocker = create_node(&app, &token, json!({"kind": "idea", "title": "blocker"})).await;
    add_edge(&app, &token, &blocked, &blocker, "requires").await;
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "good", "status": "active", "focus": "primary"}),
    )
    .await;
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "finished", "status": "done", "focus": "primary"}),
    )
    .await;
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "secondary", "status": "active", "focus": "secondary"}),
    )
    .await;
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "paused", "status": "paused", "focus": "primary"}),
    )
    .await;
    // A path is active when something inside it is, but never listed here.
    let path = create_node(
        &app,
        &token,
        json!({"kind": "path", "title": "path", "focus": "background"}),
    )
    .await;
    let inside = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "inside", "status": "active", "focus": "background"}),
    )
    .await;
    add_edge(&app, &token, &inside, &path, "part_of").await;

    let (_, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;

    assert_eq!(
        titles(&dashboard["active"]),
        vec!["good", "blocked", "secondary", "inside"]
    );
    assert_eq!(dashboard["active"][1]["blocked"], true);
    assert_eq!(dashboard["counts"]["blocked"], 1);
}

#[sqlx::test]
async fn ideas_count_and_recent_ideas_cover_every_idea_kind_node(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "ideas@example.com").await;

    for index in 0..7 {
        create_node(
            &app,
            &token,
            json!({"kind": "idea", "title": format!("idea-{index}")}),
        )
        .await;
    }
    // A project at status idea is committed work, not an idea.
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "project", "status": "idea"}),
    )
    .await;

    let (_, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;

    assert_eq!(dashboard["counts"]["ideas"], 7);
    assert_eq!(
        titles(&dashboard["recent_ideas"]),
        vec!["idea-6", "idea-5", "idea-4", "idea-3", "idea-2"]
    );
}

#[sqlx::test]
async fn paths_list_open_path_kind_nodes_with_progress(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "paths@example.com").await;

    let path = create_node(&app, &token, json!({"kind": "path", "title": "path"})).await;
    let done_child = create_node(
        &app,
        &token,
        json!({"kind": "study", "title": "child-done", "status": "done"}),
    )
    .await;
    let open_child = create_node(
        &app,
        &token,
        json!({"kind": "study", "title": "child-open"}),
    )
    .await;
    add_edge(&app, &token, &done_child, &path, "part_of").await;
    add_edge(&app, &token, &open_child, &path, "part_of").await;

    let archived_path = create_node(
        &app,
        &token,
        json!({"kind": "path", "title": "archived-path"}),
    )
    .await;
    // Archived itself, whatever the status derived from what's inside.
    send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{archived_path}"),
            json!({"archived": true}),
            Some(&token),
        ),
    )
    .await;

    let (_, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;

    assert_eq!(titles(&dashboard["paths"]), vec!["path".to_string()]);
    assert_eq!(
        dashboard["paths"][0]["container_progress"],
        json!({"done": 1, "total": 2})
    );
}

#[sqlx::test]
async fn dashboard_is_isolated_per_user(pool: PgPool) {
    let app = app(pool.clone());
    let token_a = signup(&app, "dashowner@example.com").await;
    let token_b = signup(&app, "dashintruder@example.com").await;
    create_node(&app, &token_a, json!({"kind": "idea", "title": "mine"})).await;
    create_node(
        &app,
        &token_a,
        json!({"kind": "project", "title": "mine-active", "status": "active"}),
    )
    .await;

    let (_, dashboard_b) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token_b)),
    )
    .await;
    assert_eq!(dashboard_b["counts"]["total"], 0);
    assert_eq!(dashboard_b["stale"].as_array().unwrap().len(), 0);
    for list in ["active", "recent_ideas", "paths"] {
        assert_eq!(dashboard_b[list].as_array().unwrap().len(), 0, "{list}");
    }
}

#[sqlx::test]
async fn missing_or_garbage_token_returns_401(pool: PgPool) {
    let app = app(pool);

    let (missing_status, _) = send(&app, req("GET", "/api/dashboard", Value::Null, None)).await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

    let (garbage_status, _) = send(
        &app,
        req(
            "GET",
            "/api/dashboard",
            Value::Null,
            Some("not-a-real-token"),
        ),
    )
    .await;
    assert_eq!(garbage_status, StatusCode::UNAUTHORIZED);
}
