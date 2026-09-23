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

async fn create_node(app: &axum::Router, token: &str, body: Value) -> Value {
    let (status, body) = send(app, req("POST", "/api/nodes", body, Some(token))).await;
    assert_eq!(status, StatusCode::CREATED);
    body
}

#[sqlx::test]
async fn create_get_update_delete_happy_path(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "crud@example.com").await;

    let node = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "Loom backend"}),
    )
    .await;
    assert_eq!(node["status"], "idea");
    assert_eq!(node["focus"], "secondary");
    assert_eq!(node["topic_ids"], json!([]));
    let node_id = node["id"].as_str().unwrap().to_owned();

    let (get_status, get_body) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_eq!(get_body["title"], "Loom backend");

    let (patch_status, patch_body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"status": "active", "progress_current": 3, "progress_total": 10}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(patch_status, StatusCode::OK);
    assert_eq!(patch_body["status"], "active");
    assert_eq!(patch_body["progress_current"], 3);
    assert_eq!(patch_body["title"], "Loom backend"); // untouched field kept

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

    let (get_after_delete, _) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(get_after_delete, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn patch_omitted_field_kept_explicit_null_clears(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "patch@example.com").await;
    let node = create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "t", "notes": "some notes"}),
    )
    .await;
    let node_id = node["id"].as_str().unwrap().to_owned();

    // Omitting `notes` entirely must leave it unchanged.
    let (_, body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"title": "renamed"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(body["notes"], "some notes");
    assert_eq!(body["title"], "renamed");

    // Explicit null must clear it.
    let (_, body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"notes": null}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(body["notes"], Value::Null);
}

#[sqlx::test]
async fn list_filters_by_status_focus_kind(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "list@example.com").await;

    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "a", "status": "active", "focus": "primary"}),
    )
    .await;
    create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "b", "status": "queued", "focus": "primary"}),
    )
    .await;

    let (_, all) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token))).await;
    assert_eq!(all.as_array().unwrap().len(), 2);

    let (_, active_only) = send(
        &app,
        req("GET", "/api/nodes?status=active", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(active_only.as_array().unwrap().len(), 1);
    assert_eq!(active_only[0]["title"], "a");

    let (_, combined) = send(
        &app,
        req(
            "GET",
            "/api/nodes?focus=primary&kind=idea",
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(combined.as_array().unwrap().len(), 1);
    assert_eq!(combined[0]["title"], "b");
}

#[sqlx::test]
async fn mismatched_progress_pair_returns_400(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "progress@example.com").await;

    let (status, _) = send(
        &app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "idea", "title": "bad", "progress_current": 5}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn attach_and_detach_topic_are_idempotent(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "linking@example.com").await;
    let node = create_node(&app, &token, json!({"kind": "idea", "title": "n"})).await;
    let node_id = node["id"].as_str().unwrap().to_owned();

    let (_, topic) = send(
        &app,
        req("POST", "/api/topics", json!({"name": "rust"}), Some(&token)),
    )
    .await;
    let topic_id = topic["id"].as_str().unwrap().to_owned();

    for _ in 0..2 {
        let (status, _) = send(
            &app,
            req(
                "POST",
                &format!("/api/nodes/{node_id}/topics"),
                json!({"topic_id": topic_id}),
                Some(&token),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    let (_, node_body) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(node_body["topic_ids"], json!([topic_id]));

    for _ in 0..2 {
        let (status, _) = send(
            &app,
            req(
                "DELETE",
                &format!("/api/nodes/{node_id}/topics/{topic_id}"),
                Value::Null,
                Some(&token),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    let (_, node_body) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(node_body["topic_ids"], json!([]));
}

#[sqlx::test]
async fn ownership_scoping_returns_404_for_another_users_node(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "owner@example.com").await;
    let token_b = signup(&app, "intruder@example.com").await;

    let node = create_node(&app, &token_a, json!({"kind": "idea", "title": "mine"})).await;
    let node_id = node["id"].as_str().unwrap().to_owned();

    let (get_status, _) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);

    let (patch_status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"title": "hijacked"}),
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(patch_status, StatusCode::NOT_FOUND);

    let (delete_status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NOT_FOUND);

    let list_only_returns_owners =
        send(&app, req("GET", "/api/nodes", Value::Null, Some(&token_b)))
            .await
            .1;
    assert_eq!(list_only_returns_owners.as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn attach_topic_belonging_to_another_user_returns_404(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "topicowner@example.com").await;
    let token_b = signup(&app, "topicintruder@example.com").await;

    let node = create_node(&app, &token_a, json!({"kind": "idea", "title": "n"})).await;
    let node_id = node["id"].as_str().unwrap().to_owned();

    let (_, topic_b) = send(
        &app,
        req(
            "POST",
            "/api/topics",
            json!({"name": "not-yours"}),
            Some(&token_b),
        ),
    )
    .await;
    let topic_b_id = topic_b["id"].as_str().unwrap().to_owned();

    // user A tries to attach user B's topic to user A's own node.
    let (status, _) = send(
        &app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/topics"),
            json!({"topic_id": topic_b_id}),
            Some(&token_a),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn attach_and_detach_topic_on_another_users_node_returns_404(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "nodeowner@example.com").await;
    let token_b = signup(&app, "nodeintruder@example.com").await;

    let node = create_node(&app, &token_a, json!({"kind": "idea", "title": "n"})).await;
    let node_id = node["id"].as_str().unwrap().to_owned();

    let (_, topic_b) = send(
        &app,
        req(
            "POST",
            "/api/topics",
            json!({"name": "b-topic"}),
            Some(&token_b),
        ),
    )
    .await;
    let topic_b_id = topic_b["id"].as_str().unwrap().to_owned();

    let (attach_status, _) = send(
        &app,
        req(
            "POST",
            &format!("/api/nodes/{node_id}/topics"),
            json!({"topic_id": topic_b_id}),
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(attach_status, StatusCode::NOT_FOUND);

    let (detach_status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/nodes/{node_id}/topics/{topic_b_id}"),
            Value::Null,
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(detach_status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn view_backlog_includes_only_unpromoted_ideas(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "viewbacklog@example.com").await;

    create_node(&app, &token, json!({"kind": "idea", "title": "backlog"})).await;
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "promoted"}),
    )
    .await;
    create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "queued-idea", "status": "queued"}),
    )
    .await;

    let (_, list) = send(
        &app,
        req("GET", "/api/nodes?view=backlog", Value::Null, Some(&token)),
    )
    .await;
    let titles: Vec<String> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["title"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(titles, vec!["backlog".to_string()]);
}

#[sqlx::test]
async fn view_archived_includes_only_archived(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "viewarchived@example.com").await;

    create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "archived", "status": "archived"}),
    )
    .await;
    create_node(&app, &token, json!({"kind": "idea", "title": "active"})).await;

    let (_, list) = send(
        &app,
        req("GET", "/api/nodes?view=archived", Value::Null, Some(&token)),
    )
    .await;
    let titles: Vec<String> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["title"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(titles, vec!["archived".to_string()]);
}

#[sqlx::test]
async fn view_all_matches_unfiltered_behavior(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "viewall@example.com").await;

    create_node(&app, &token, json!({"kind": "idea", "title": "a"})).await;
    create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "b", "status": "archived"}),
    )
    .await;

    let (_, unfiltered) = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token))).await;
    let (_, view_all) = send(
        &app,
        req("GET", "/api/nodes?view=all", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(
        unfiltered.as_array().unwrap().len(),
        view_all.as_array().unwrap().len()
    );
    assert_eq!(unfiltered.as_array().unwrap().len(), 2);
}

#[sqlx::test]
async fn view_combines_with_explicit_status_filter(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "viewcombine@example.com").await;
    create_node(&app, &token, json!({"kind": "idea", "title": "backlog"})).await;

    // view=backlog implies status=idea; contradicting it with status=active
    // legitimately empties the result via AND, not a special case.
    let (_, list) = send(
        &app,
        req(
            "GET",
            "/api/nodes?view=backlog&status=active",
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(list.as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn missing_or_garbage_token_returns_401(pool: PgPool) {
    let app = app(pool);

    let (missing_status, _) = send(&app, req("GET", "/api/nodes", Value::Null, None)).await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

    let (garbage_status, _) = send(
        &app,
        req("GET", "/api/nodes", Value::Null, Some("not-a-real-token")),
    )
    .await;
    assert_eq!(garbage_status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn empty_title_returns_400(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "emptytitle@example.com").await;

    let (create_status, _) = send(
        &app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "idea", "title": "   "}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(create_status, StatusCode::BAD_REQUEST);

    let node = create_node(&app, &token, json!({"kind": "idea", "title": "real"})).await;
    let node_id = node["id"].as_str().unwrap();

    let (update_status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"title": ""}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(update_status, StatusCode::BAD_REQUEST);
}
