use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt;
use loom::app::build_router;
use loom::state::AppState;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test-only-secret";

const TEST_PEER: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    12345,
);

fn app(pool: PgPool) -> axum::Router {
    build_router(AppState::new(pool, TEST_JWT_SECRET).with_mcp("http://localhost:8081"))
}

async fn send(app: &axum::Router, req: Request<Body>) -> (StatusCode, HeaderMap, Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

fn req(method: &str, uri: &str, body: Value, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::HOST, "localhost:8081")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream")
        .extension(ConnectInfo(TEST_PEER));
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn signup(app: &axum::Router, email: &str) -> String {
    let (status, _, body) = send(
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

/// Signs up and creates a personal MCP token; returns (web JWT, MCP token).
async fn user_with_token(app: &axum::Router, email: &str) -> (String, String) {
    let jwt = signup(app, email).await;
    let (status, _, body) = send(
        app,
        req(
            "POST",
            "/api/mcp/tokens",
            json!({"name": "test"}),
            Some(&jwt),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    (jwt, body["raw_token"].as_str().unwrap().to_owned())
}

async fn rpc(
    app: &axum::Router,
    token: Option<&str>,
    method: &str,
    params: Value,
) -> (StatusCode, HeaderMap, Value) {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params});
    send(app, req("POST", "/mcp", body, token)).await
}

/// Calls a tool and returns its `result` (a CallToolResult).
async fn call(app: &axum::Router, token: &str, tool: &str, arguments: Value) -> Value {
    let (status, _, body) = rpc(
        app,
        Some(token),
        "tools/call",
        json!({"name": tool, "arguments": arguments}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["result"].clone()
}

fn is_error(result: &Value) -> bool {
    result["isError"] == json!(true)
}

fn error_text(result: &Value) -> String {
    result["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[sqlx::test]
async fn rejects_missing_invalid_and_web_session_tokens(pool: PgPool) {
    let app = app(pool);
    let (jwt, _) = user_with_token(&app, "auth@example.com").await;

    for token in [None, Some("loom_not-a-real-token"), Some(jwt.as_str())] {
        let (status, headers, _) = rpc(&app, token, "tools/list", json!({})).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(headers.contains_key(header::WWW_AUTHENTICATE));
    }
}

#[sqlx::test]
async fn revoked_token_stops_working(pool: PgPool) {
    let app = app(pool);
    let (jwt, token) = user_with_token(&app, "revoke@example.com").await;
    let (status, _, _) = rpc(&app, Some(&token), "tools/list", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    let (_, _, tokens) = send(&app, req("GET", "/api/mcp/tokens", Value::Null, Some(&jwt))).await;
    let token_id = tokens[0]["id"].as_str().unwrap();
    let uri = format!("/api/mcp/tokens/{token_id}");
    send(&app, req("DELETE", &uri, Value::Null, Some(&jwt))).await;

    let (status, _, _) = rpc(&app, Some(&token), "tools/list", json!({})).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn disabled_by_default(pool: PgPool) {
    let app = build_router(AppState::new(pool, TEST_JWT_SECRET));
    let (status, _, _) = rpc(&app, Some("anything"), "tools/list", json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn lists_tools(pool: PgPool) {
    let app = app(pool);
    let (_, token) = user_with_token(&app, "tools@example.com").await;
    let (status, _, body) = rpc(&app, Some(&token), "tools/list", json!({})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let names: Vec<&str> = body["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    for expected in [
        "loom_get_graph",
        "loom_create_node",
        "loom_create_subgraph",
        "loom_update_node",
    ] {
        assert!(names.contains(&expected), "missing {expected} in {names:?}");
    }
}

#[sqlx::test]
async fn create_subgraph_builds_a_learning_path(pool: PgPool) {
    let app = app(pool);
    let (jwt, token) = user_with_token(&app, "path@example.com").await;

    let result = call(
        &app,
        &token,
        "loom_create_subgraph",
        json!({
            "nodes": [
                {"ref": "path", "kind": "path", "title": "Learn Rust async"},
                {"ref": "book", "kind": "study", "title": "Async book", "progress_current": 0,
                 "progress_total": 12, "progress_unit": "chapters", "notes": "Read it **all**"},
                {"ref": "demo", "kind": "project", "title": "Build a chat server",
                 "checklist": ["Accept connections", "Broadcast messages"]}
            ],
            "edges": [
                {"from": "book", "to": "path", "kind": "part_of"},
                {"from": "demo", "to": "path", "kind": "part_of"},
                {"from": "demo", "to": "book", "kind": "requires"}
            ]
        }),
    )
    .await;
    assert!(!is_error(&result), "{result}");
    let ids = &result["structuredContent"]["ids"];
    let demo_id = ids["demo"].as_str().unwrap();

    let (_, _, canvas) = send(
        &app,
        req("GET", "/api/board/canvas", Value::Null, Some(&jwt)),
    )
    .await;
    assert_eq!(canvas["nodes"].as_array().unwrap().len(), 3);
    assert_eq!(canvas["edges"].as_array().unwrap().len(), 3);

    let details = call(&app, &token, "loom_get_node", json!({"node_id": demo_id})).await;
    let details = &details["structuredContent"];
    assert_eq!(details["node"]["blocked"], json!(true));
    assert_eq!(details["checklist"].as_array().unwrap().len(), 2);
    assert!(details["node"].get("canvas_x").is_none());

    let path = call(
        &app,
        &token,
        "loom_get_node",
        json!({"node_id": ids["path"]}),
    )
    .await;
    assert_eq!(
        path["structuredContent"]["node"]["container_progress"],
        json!({"done": 0, "total": 2})
    );
}

#[sqlx::test]
async fn failing_subgraph_leaves_nothing_behind(pool: PgPool) {
    let app = app(pool);
    let (jwt, token) = user_with_token(&app, "rollback@example.com").await;

    let result = call(
        &app,
        &token,
        "loom_create_subgraph",
        json!({
            "nodes": [
                {"ref": "a", "kind": "study", "title": "A"},
                {"ref": "b", "kind": "study", "title": "B"}
            ],
            "edges": [
                {"from": "a", "to": "b", "kind": "requires"},
                {"from": "b", "to": "a", "kind": "requires"}
            ]
        }),
    )
    .await;
    assert!(is_error(&result));
    assert!(error_text(&result).contains("edges[1]"), "{result}");
    assert!(error_text(&result).contains("cycle"), "{result}");

    let (_, _, canvas) = send(
        &app,
        req("GET", "/api/board/canvas", Value::Null, Some(&jwt)),
    )
    .await;
    assert!(canvas["nodes"].as_array().unwrap().is_empty());
    assert!(canvas["edges"].as_array().unwrap().is_empty());
}

#[sqlx::test]
async fn tools_reuse_handler_validation(pool: PgPool) {
    let app = app(pool);
    let (jwt, token) = user_with_token(&app, "validate@example.com").await;

    let result = call(
        &app,
        &token,
        "loom_create_node",
        json!({"kind": "idea", "title": "Nope", "checklist": ["x"]}),
    )
    .await;
    assert!(is_error(&result));
    assert!(error_text(&result).contains("only projects have a checklist"));

    let result = call(
        &app,
        &token,
        "loom_create_node",
        json!({"kind": "idea", "title": "Nope", "progress_total": 3}),
    )
    .await;
    assert!(error_text(&result).contains("only study nodes track progress"));

    let (_, _, nodes) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&jwt))).await;
    assert!(nodes.as_array().unwrap().is_empty());
}

#[sqlx::test]
async fn cannot_touch_another_users_nodes(pool: PgPool) {
    let app = app(pool);
    let (owner_jwt, owner) = user_with_token(&app, "owner@example.com").await;
    let (_, intruder) = user_with_token(&app, "intruder@example.com").await;

    let created = call(
        &app,
        &owner,
        "loom_create_node",
        json!({"kind": "project", "title": "Secret"}),
    )
    .await;
    let node_id = created["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_owned();

    let listed = call(&app, &intruder, "loom_list_nodes", json!({})).await;
    assert!(
        listed["structuredContent"]["nodes"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    for (tool, arguments) in [
        ("loom_get_node", json!({"node_id": node_id})),
        (
            "loom_update_node",
            json!({"node_id": node_id, "title": "Mine now"}),
        ),
        (
            "loom_add_checklist_items",
            json!({"node_id": node_id, "titles": ["x"]}),
        ),
        ("loom_poke_node", json!({"node_id": node_id})),
        ("loom_delete_node", json!({"node_id": node_id})),
    ] {
        let result = call(&app, &intruder, tool, arguments).await;
        assert!(is_error(&result), "{tool} succeeded: {result}");
        assert!(
            error_text(&result).contains("not found"),
            "{tool}: {result}"
        );
    }

    let uri = format!("/api/nodes/{node_id}");
    let (status, _, node) = send(&app, req("GET", &uri, Value::Null, Some(&owner_jwt))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(node["title"], "Secret");
}
