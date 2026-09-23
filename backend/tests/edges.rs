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

async fn create_edge(
    app: &axum::Router,
    token: &str,
    from: &str,
    to: &str,
    kind: &str,
) -> (StatusCode, Value) {
    send(
        app,
        req(
            "POST",
            "/api/edges",
            json!({"from_node_id": from, "to_node_id": to, "kind": kind}),
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

async fn set_status(app: &axum::Router, token: &str, id: &str, status: &str) {
    let (status_code, _) = send(
        app,
        req(
            "PATCH",
            &format!("/api/nodes/{id}"),
            json!({"status": status}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status_code, StatusCode::OK);
}

#[sqlx::test]
async fn create_list_delete_happy_path(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "edgecrud@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    let (status, edge) = create_edge(&app, &token, &a, &b, "related").await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(edge["from_node_id"], a);
    assert_eq!(edge["to_node_id"], b);
    assert_eq!(edge["kind"], "related");
    let edge_id = edge["id"].as_str().unwrap().to_owned();

    let (_, all) = send(&app, req("GET", "/api/edges", Value::Null, Some(&token))).await;
    assert_eq!(all.as_array().unwrap().len(), 1);

    let (_, by_from) = send(
        &app,
        req(
            "GET",
            &format!("/api/edges?from_node_id={a}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(by_from.as_array().unwrap().len(), 1);

    let (_, by_kind_mismatch) = send(
        &app,
        req("GET", "/api/edges?kind=requires", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(by_kind_mismatch.as_array().unwrap().len(), 0);

    let (delete_status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/edges/{edge_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NO_CONTENT);

    let (delete_again, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/edges/{edge_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(delete_again, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn create_edge_ownership_scoping(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "edgeowner@example.com").await;
    let token_b = signup(&app, "edgeintruder@example.com").await;

    let node1 = create_node(&app, &token_a, "node1").await;
    let node2 = create_node(&app, &token_b, "node2").await;

    let (status, _) = create_edge(&app, &token_a, &node1, &node2, "related").await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = create_edge(&app, &token_a, &node2, &node1, "related").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn requires_cycle_rejected_two_node(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "cycle2@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    let (status, _) = create_edge(&app, &token, &a, &b, "requires").await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, _) = create_edge(&app, &token, &b, &a, "requires").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn requires_cycle_rejected_three_node(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "cycle3@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;
    let c = create_node(&app, &token, "C").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "requires").await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        create_edge(&app, &token, &b, &c, "requires").await.0,
        StatusCode::CREATED
    );

    let (status, _) = create_edge(&app, &token, &c, &a, "requires").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn duplicate_edge_returns_409(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "dupedge@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "related").await.0,
        StatusCode::CREATED
    );
    let (status, _) = create_edge(&app, &token, &a, &b, "related").await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn self_loop_returns_400(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "selfloop@example.com").await;
    let a = create_node(&app, &token, "A").await;

    let (status, _) = create_edge(&app, &token, &a, &a, "related").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn blocked_reflects_requires_target_status(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "blocked@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;

    assert_eq!(
        create_edge(&app, &token, &a, &b, "requires").await.0,
        StatusCode::CREATED
    );

    let node_a = get_node(&app, &token, &a).await;
    assert_eq!(node_a["blocked"], true);

    // PATCH's derived_state follow-up recomputes A even though A itself
    // wasn't the node being patched.
    set_status(&app, &token, &b, "done").await;

    let node_a = get_node(&app, &token, &a).await;
    assert_eq!(node_a["blocked"], false);
}

#[sqlx::test]
async fn container_progress_reflects_children(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "container@example.com").await;
    let p = create_node(&app, &token, "P").await;
    let x = create_node(&app, &token, "X").await;
    let y = create_node(&app, &token, "Y").await;

    let no_children = get_node(&app, &token, &p).await;
    assert_eq!(no_children["container_progress"], Value::Null);

    assert_eq!(
        create_edge(&app, &token, &x, &p, "part_of").await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        create_edge(&app, &token, &y, &p, "part_of").await.0,
        StatusCode::CREATED
    );

    let container = get_node(&app, &token, &p).await;
    assert_eq!(
        container["container_progress"],
        json!({"done": 0, "total": 2})
    );

    set_status(&app, &token, &x, "done").await;

    let container = get_node(&app, &token, &p).await;
    assert_eq!(
        container["container_progress"],
        json!({"done": 1, "total": 2})
    );
}

#[sqlx::test]
async fn list_nodes_agrees_with_get_node_on_derived_fields(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "consistency@example.com").await;
    let a = create_node(&app, &token, "A").await;
    let b = create_node(&app, &token, "B").await;
    assert_eq!(
        create_edge(&app, &token, &a, &b, "requires").await.0,
        StatusCode::CREATED
    );

    let via_get = get_node(&app, &token, &a).await;
    let (_, list) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token))).await;
    let via_list = list
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["id"] == a)
        .unwrap();

    assert_eq!(via_get["blocked"], via_list["blocked"]);
    assert_eq!(
        via_get["container_progress"],
        via_list["container_progress"]
    );
}
