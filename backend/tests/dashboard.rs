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

async fn backdate_created_at(pool: &PgPool, node_id: &str, days: i32) {
    let node_id: uuid::Uuid = node_id.parse().unwrap();
    sqlx::query!(
        "UPDATE nodes SET created_at = now() - make_interval(days => $2) WHERE id = $1",
        node_id,
        days,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn poke_and_backdate(
    app: &axum::Router,
    pool: &PgPool,
    token: &str,
    node_id: &str,
    days: i32,
) {
    let (status, _) = send(
        app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/pokes"),
            Value::Null,
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let node_uuid: uuid::Uuid = node_id.parse().unwrap();
    sqlx::query!(
        "UPDATE pokes SET poked_at = now() - make_interval(days => $2) WHERE node_id = $1",
        node_uuid,
        days,
    )
    .execute(pool)
    .await
    .unwrap();
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
        json!({"kind": "course", "title": "d", "status": "archived"}),
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
    assert_eq!(counts["by_status"]["idea"], 2);
    assert_eq!(counts["by_status"]["active"], 1);
    assert_eq!(counts["by_status"]["archived"], 1);
    assert_eq!(counts["by_status"]["queued"], 0);
    assert_eq!(counts["by_kind"]["idea"], 2);
    assert_eq!(counts["by_kind"]["project"], 1);
    assert_eq!(counts["by_kind"]["course"], 1);
    assert_eq!(counts["backlog"], 2);
    assert_eq!(counts["blocked"], 0);
}

#[sqlx::test]
async fn stale_list_reflects_14_day_threshold(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "stale@example.com").await;

    let never_poked_old = create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "never-poked-old", "status": "active"}),
    )
    .await;
    backdate_created_at(&pool, &never_poked_old, 15).await;

    let poked_long_ago = create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "poked-long-ago", "status": "active"}),
    )
    .await;
    poke_and_backdate(&app, &pool, &token, &poked_long_ago, 15).await;

    let recently_poked = create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "recently-poked", "status": "active"}),
    )
    .await;
    backdate_created_at(&pool, &recently_poked, 20).await;
    let (status, _) = send(
        &app,
        req(
            "POST",
            &format!("/api/nodes/{recently_poked}/pokes"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let stale_but_done = create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "stale-but-done", "status": "done"}),
    )
    .await;
    backdate_created_at(&pool, &stale_but_done, 20).await;

    let (_, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;
    let stale_titles: Vec<String> = dashboard["stale"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["title"].as_str().unwrap().to_owned())
        .collect();

    assert!(stale_titles.contains(&"never-poked-old".to_string()));
    assert!(stale_titles.contains(&"poked-long-ago".to_string()));
    assert!(!stale_titles.contains(&"recently-poked".to_string()));
    assert!(!stale_titles.contains(&"stale-but-done".to_string()));
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
async fn primary_list_includes_blocked_and_ranks_actionable_first(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "primary@example.com").await;

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

    let (_, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;

    assert_eq!(
        titles(&dashboard["primary"]),
        vec![
            "good".to_string(),
            "blocked".to_string(),
            "queued".to_string()
        ]
    );
    assert_eq!(dashboard["primary"][1]["blocked"], true);
    assert_eq!(dashboard["counts"]["blocked"], 1);
}

#[sqlx::test]
async fn backlog_count_and_recent_backlog_cover_only_unpromoted_ideas(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "backlog@example.com").await;

    for index in 0..7 {
        create_node(
            &app,
            &token,
            json!({"kind": "idea", "title": format!("idea-{index}")}),
        )
        .await;
    }
    // A queued idea and a project at status=idea are not backlog.
    create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "queued-idea", "status": "queued"}),
    )
    .await;
    create_node(&app, &token, json!({"kind": "project", "title": "project"})).await;

    let (_, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;

    assert_eq!(dashboard["counts"]["backlog"], 7);
    assert_eq!(
        titles(&dashboard["recent_backlog"]),
        vec!["idea-6", "idea-5", "idea-4", "idea-3", "idea-2"]
    );
}

#[sqlx::test]
async fn containers_list_open_paths_with_progress(pool: PgPool) {
    let app = app(pool.clone());
    let token = signup(&app, "containers@example.com").await;

    let path = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "path", "status": "active"}),
    )
    .await;
    let done_child = create_node(
        &app,
        &token,
        json!({"kind": "course", "title": "child-done", "status": "done"}),
    )
    .await;
    let open_child = create_node(
        &app,
        &token,
        json!({"kind": "course", "title": "child-open"}),
    )
    .await;
    add_edge(&app, &token, &done_child, &path, "part_of").await;
    add_edge(&app, &token, &open_child, &path, "part_of").await;

    let archived_path = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "archived-path", "status": "archived"}),
    )
    .await;
    add_edge(&app, &token, &open_child, &archived_path, "part_of").await;

    let (_, dashboard) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token)),
    )
    .await;

    assert_eq!(titles(&dashboard["containers"]), vec!["path".to_string()]);
    assert_eq!(
        dashboard["containers"][0]["container_progress"],
        json!({"done": 1, "total": 2})
    );
}

#[sqlx::test]
async fn dashboard_is_isolated_per_user(pool: PgPool) {
    let app = app(pool.clone());
    let token_a = signup(&app, "dashowner@example.com").await;
    let token_b = signup(&app, "dashintruder@example.com").await;
    create_node(&app, &token_a, json!({"kind": "idea", "title": "mine"})).await;

    let (_, dashboard_b) = send(
        &app,
        req("GET", "/api/dashboard", Value::Null, Some(&token_b)),
    )
    .await;
    assert_eq!(dashboard_b["counts"]["total"], 0);
    assert_eq!(dashboard_b["stale"].as_array().unwrap().len(), 0);
    for list in ["primary", "recent_backlog", "containers"] {
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
