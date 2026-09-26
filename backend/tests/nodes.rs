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
        json!({"kind": "study", "title": "Loom backend"}),
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
async fn started_at_and_completed_at_are_auto_stamped_and_cleared(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "autostamp@example.com").await;
    let node = create_node(&app, &token, json!({"kind": "project", "title": "t"})).await;
    let node_id = node["id"].as_str().unwrap().to_owned();
    assert_eq!(node["started_at"], Value::Null);
    assert_eq!(node["completed_at"], Value::Null);

    // status -> active stamps started_at, once, automatically.
    let (_, body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"status": "active"}),
            Some(&token),
        ),
    )
    .await;
    assert!(!body["started_at"].is_null());
    let started_at = body["started_at"].clone();

    // An unrelated update doesn't re-stamp it.
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
    assert_eq!(body["started_at"], started_at);

    // status -> done stamps completed_at.
    let (_, body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"status": "done"}),
            Some(&token),
        ),
    )
    .await;
    assert!(!body["completed_at"].is_null());

    // Moving away from done clears completed_at again.
    let (_, body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"status": "active"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(body["completed_at"], Value::Null);
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
            json!({"kind": "study", "title": "bad", "progress_current": 5}),
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
async fn reorder_sets_ranks_and_rejects_another_users_node(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "orderer@example.com").await;
    let token_b = signup(&app, "orderintruder@example.com").await;

    let first = create_node(&app, &token_a, json!({"kind": "idea", "title": "first"})).await;
    let second = create_node(&app, &token_a, json!({"kind": "idea", "title": "second"})).await;
    let foreign = create_node(
        &app,
        &token_b,
        json!({"kind": "idea", "title": "not yours"}),
    )
    .await;

    let (status, _) = send(
        &app,
        req(
            "PUT",
            "/api/nodes/order",
            json!({"node_ids": [second["id"], first["id"]]}),
            Some(&token_a),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let after = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token_a)))
        .await
        .1;
    let by_id = |id: &Value| {
        after
            .as_array()
            .unwrap()
            .iter()
            .find(|node| &node["id"] == id)
            .unwrap()
    };
    assert_eq!(by_id(&second["id"])["sort_order"], 1);
    assert_eq!(by_id(&first["id"])["sort_order"], 2);

    // Including another user's node rolls the whole write back.
    let (rejected_status, _) = send(
        &app,
        req(
            "PUT",
            "/api/nodes/order",
            json!({"node_ids": [first["id"], foreign["id"]]}),
            Some(&token_a),
        ),
    )
    .await;
    assert_eq!(rejected_status, StatusCode::NOT_FOUND);
    let unchanged = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token_a)))
        .await
        .1;
    let unchanged_first = unchanged
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"] == first["id"])
        .unwrap();
    assert_eq!(unchanged_first["sort_order"], 2);

    let (clear_status, _) = send(
        &app,
        req("DELETE", "/api/nodes/order", Value::Null, Some(&token_a)),
    )
    .await;
    assert_eq!(clear_status, StatusCode::NO_CONTENT);
    let cleared = send(&app, req("GET", "/api/nodes", Value::Null, Some(&token_a)))
        .await
        .1;
    for node in cleared.as_array().unwrap() {
        assert_eq!(node["sort_order"], Value::Null);
    }
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
async fn view_backlog_includes_every_status_idea_node(pool: PgPool) {
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
    // The project was created at the default status (idea), so it's backlog
    // too; the queued idea isn't.
    assert_eq!(titles, vec!["promoted".to_string(), "backlog".to_string()]);
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
async fn malformed_path_uuid_returns_400_with_consistent_error_shape(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "badpathuuid@example.com").await;

    let (status, body) = send(
        &app,
        req("GET", "/api/nodes/not-a-uuid", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"].is_string(),
        "expected {{\"error\": ...}} shape, got {body:?}"
    );
}

#[sqlx::test]
async fn color_must_be_a_hex_code(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "badcolor@example.com").await;

    let (status, _) = send(
        &app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "idea", "title": "n", "color": "blue"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let node = create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "n", "color": "#a3c9ff"}),
    )
    .await;
    assert_eq!(node["color"], "#a3c9ff");
}

#[sqlx::test]
async fn title_and_color_are_trimmed(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "trimmed@example.com").await;

    let node = create_node(
        &app,
        &token,
        json!({"kind": "idea", "title": "  padded title  "}),
    )
    .await;
    assert_eq!(node["title"], "padded title");
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

#[sqlx::test]
async fn canvas_position_is_null_until_set_and_round_trips(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "canvas@example.com").await;
    let node = create_node(&app, &token, json!({"kind": "idea", "title": "placed"})).await;
    assert_eq!(node["canvas_x"], Value::Null);
    assert_eq!(node["canvas_y"], Value::Null);
    let node_id = node["id"].as_str().unwrap();
    let uri = format!("/api/nodes/{node_id}");

    let (status, placed) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"canvas_x": 120.5, "canvas_y": -40}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(placed["canvas_x"], 120.5);
    assert_eq!(placed["canvas_y"], -40.0);

    // A PATCH that doesn't mention the position leaves it alone.
    let (_, renamed) = send(
        &app,
        req("PATCH", &uri, json!({"title": "renamed"}), Some(&token)),
    )
    .await;
    assert_eq!(renamed["canvas_x"], 120.5);

    let (_, canvas) = send(
        &app,
        req("GET", "/api/board/canvas", Value::Null, Some(&token)),
    )
    .await;
    assert_eq!(canvas["nodes"][0]["canvas_y"], -40.0);

    let (status, cleared) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"canvas_x": null, "canvas_y": null}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cleared["canvas_x"], Value::Null);
}

#[sqlx::test]
async fn half_set_canvas_position_returns_400(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "halfcanvas@example.com").await;
    let node = create_node(&app, &token, json!({"kind": "idea", "title": "placed"})).await;
    let node_id = node["id"].as_str().unwrap();

    let (status, body) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"canvas_x": 10}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body["error"],
        "canvas_x/canvas_y must be set or cleared together"
    );
}

#[sqlx::test]
async fn creating_straight_into_active_starts_it(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "createstamp@example.com").await;

    let active = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "a", "status": "active"}),
    )
    .await;
    assert!(!active["started_at"].is_null());
    assert_eq!(active["completed_at"], Value::Null);

    // Done without ever being active: no start, so no completion date.
    let done = create_node(
        &app,
        &token,
        json!({"kind": "study", "title": "d", "status": "done"}),
    )
    .await;
    assert_eq!(done["started_at"], Value::Null);
    assert_eq!(done["completed_at"], Value::Null);

    let queued = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "q", "status": "queued"}),
    )
    .await;
    assert_eq!(queued["started_at"], Value::Null);
}

#[sqlx::test]
async fn explicit_started_and_completed_dates_move_the_period_edges(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "explicitdates@example.com").await;
    let node = create_node(
        &app,
        &token,
        json!({"kind": "project", "title": "t", "status": "active"}),
    )
    .await;
    let node_id = node["id"].as_str().unwrap();
    let uri = format!("/api/nodes/{node_id}");

    // A completion date only exists on a done node.
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"completed_at": "2026-02-10T00:00:00Z"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Backdating started_at and completing with an explicit date moves the
    // first period's start and the last period's end.
    let (status, body) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({
                "status": "done",
                "started_at": "2026-01-05T00:00:00Z",
                "completed_at": "2026-02-10T00:00:00Z"
            }),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["started_at"], "2026-01-05T00:00:00Z");
    assert_eq!(body["completed_at"], "2026-02-10T00:00:00Z");
    let periods = list_periods(&app, &token, node_id).await;
    assert_eq!(periods.len(), 1);
    assert_eq!(periods[0]["started_at"], "2026-01-05T00:00:00Z");
    assert_eq!(periods[0]["ended_at"], "2026-02-10T00:00:00Z");

    // A start after the period's end is rejected.
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"started_at": "2026-03-01T00:00:00Z"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Clearing started_at removes every period: not started, not completed.
    let (_, cleared) = send(
        &app,
        req("PATCH", &uri, json!({"started_at": null}), Some(&token)),
    )
    .await;
    assert_eq!(cleared["started_at"], Value::Null);
    assert_eq!(cleared["completed_at"], Value::Null);
    assert!(list_periods(&app, &token, node_id).await.is_empty());
}

#[sqlx::test]
async fn progress_unit_is_trimmed_optional_and_clearable(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "progressunit@example.com").await;
    let node = create_node(
        &app,
        &token,
        json!({
            "kind": "study", "title": "book",
            "progress_current": 3, "progress_total": 20, "progress_unit": "  chapters  "
        }),
    )
    .await;
    assert_eq!(node["progress_unit"], "chapters");
    let uri = format!("/api/nodes/{}", node["id"].as_str().unwrap());

    // Untouched by an unrelated PATCH, blank clears it.
    let (_, renamed) = send(
        &app,
        req("PATCH", &uri, json!({"title": "b"}), Some(&token)),
    )
    .await;
    assert_eq!(renamed["progress_unit"], "chapters");
    let (_, blanked) = send(
        &app,
        req("PATCH", &uri, json!({"progress_unit": "   "}), Some(&token)),
    )
    .await;
    assert_eq!(blanked["progress_unit"], Value::Null);

    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"progress_unit": "x".repeat(41)}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn only_study_nodes_accept_tracked_progress(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "studyonly@example.com").await;

    let (status, body) = send(
        &app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "project", "title": "p", "progress_current": 1, "progress_total": 2}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "only study nodes track progress");

    let project = create_node(&app, &token, json!({"kind": "project", "title": "p"})).await;
    let uri = format!("/api/nodes/{}", project["id"].as_str().unwrap());
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"progress_unit": "videos"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Changing kind to study in the same request is fine.
    let (status, body) = send(
        &app,
        req(
            "PATCH",
            &uri,
            json!({"kind": "study", "progress_current": 1, "progress_total": 4}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["progress_total"], 4);
}

#[sqlx::test]
async fn kind_change_clears_data_the_new_kind_cannot_have(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "kindswitch@example.com").await;

    let study = create_node(
        &app,
        &token,
        json!({
            "kind": "study", "title": "s",
            "progress_current": 2, "progress_total": 9, "progress_unit": "videos"
        }),
    )
    .await;
    let study_uri = format!("/api/nodes/{}", study["id"].as_str().unwrap());
    let (_, became_project) = send(
        &app,
        req(
            "PATCH",
            &study_uri,
            json!({"kind": "project"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(became_project["progress_current"], Value::Null);
    assert_eq!(became_project["progress_total"], Value::Null);
    assert_eq!(became_project["progress_unit"], Value::Null);

    // Now a project: give it a checklist, then switch away.
    let checklist_uri = format!("/api/nodes/{}/checklist", study["id"].as_str().unwrap());
    let (status, _) = send(
        &app,
        req(
            "POST",
            &checklist_uri,
            json!({"title": "task"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (_, became_idea) = send(
        &app,
        req("PATCH", &study_uri, json!({"kind": "idea"}), Some(&token)),
    )
    .await;
    assert_eq!(became_idea["checklist_progress"], Value::Null);
    let (_, items) = send(&app, req("GET", &checklist_uri, Value::Null, Some(&token))).await;
    assert_eq!(items, json!([]));

    // And checklists can't be added to a non-project.
    let (status, body) = send(
        &app,
        req(
            "POST",
            &checklist_uri,
            json!({"title": "task"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "only projects have a checklist");
}

async fn patch_status(
    app: &axum::Router,
    token: &str,
    node_id: &str,
    status: &str,
    track: bool,
) -> Value {
    let (_, body) = send(
        app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"status": status, "track_active_periods": track}),
            Some(token),
        ),
    )
    .await;
    body
}

/// A node's periods, oldest first.
async fn list_periods(app: &axum::Router, token: &str, node_id: &str) -> Vec<Value> {
    let (_, body) = send(
        app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}/periods"),
            Value::Null,
            Some(token),
        ),
    )
    .await;
    let mut periods = body.as_array().unwrap().clone();
    periods.reverse();
    periods
}

async fn project(app: &axum::Router, token: &str) -> String {
    let node = create_node(app, token, json!({"kind": "project", "title": "t"})).await;
    node["id"].as_str().unwrap().to_owned()
}

#[sqlx::test]
async fn first_activation_opens_the_first_period(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "firstperiod@example.com").await;
    let node_id = project(&app, &token).await;
    assert!(list_periods(&app, &token, &node_id).await.is_empty());

    let active = patch_status(&app, &token, &node_id, "active", true).await;
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 1);
    assert_eq!(periods[0]["started_at"], active["started_at"]);
    assert_eq!(periods[0]["ended_at"], Value::Null);
}

#[sqlx::test]
async fn done_closes_the_period_and_its_end_is_the_completed_date(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "doneperiod@example.com").await;
    let node_id = project(&app, &token).await;

    patch_status(&app, &token, &node_id, "active", true).await;
    let done = patch_status(&app, &token, &node_id, "done", true).await;
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 1);
    assert!(!done["completed_at"].is_null());
    assert_eq!(periods[0]["ended_at"], done["completed_at"]);
    assert_eq!(periods[0]["started_at"], done["started_at"]);
}

#[sqlx::test]
async fn tracking_opens_a_new_period_on_reactivation(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "trackon@example.com").await;
    let node_id = project(&app, &token).await;

    patch_status(&app, &token, &node_id, "active", true).await;
    patch_status(&app, &token, &node_id, "paused", true).await;
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 1);
    assert!(!periods[0]["ended_at"].is_null());

    patch_status(&app, &token, &node_id, "active", true).await;
    patch_status(&app, &token, &node_id, "done", true).await;
    let reactivated = patch_status(&app, &token, &node_id, "active", true).await;
    assert_eq!(reactivated["completed_at"], Value::Null);
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 3);
    assert_eq!(periods[2]["ended_at"], Value::Null);
    // Started stays the first period's start throughout.
    assert_eq!(reactivated["started_at"], periods[0]["started_at"]);
}

#[sqlx::test]
async fn without_tracking_reactivation_reopens_the_last_period(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "trackoff@example.com").await;
    let node_id = project(&app, &token).await;

    patch_status(&app, &token, &node_id, "active", false).await;
    // Pausing without tracking leaves the period running.
    patch_status(&app, &token, &node_id, "paused", false).await;
    assert_eq!(
        list_periods(&app, &token, &node_id).await[0]["ended_at"],
        Value::Null
    );

    patch_status(&app, &token, &node_id, "done", false).await;
    let reactivated = patch_status(&app, &token, &node_id, "active", false).await;
    assert_eq!(reactivated["completed_at"], Value::Null);
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 1);
    assert_eq!(periods[0]["ended_at"], Value::Null);
}

#[sqlx::test]
async fn done_after_a_pause_adds_a_completion_day_period(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "pausedone@example.com").await;
    let node_id = project(&app, &token).await;

    patch_status(&app, &token, &node_id, "active", true).await;
    patch_status(&app, &token, &node_id, "paused", true).await;
    let paused_end = list_periods(&app, &token, &node_id).await[0]["ended_at"].clone();

    let done = patch_status(&app, &token, &node_id, "done", true).await;
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 2);
    // The pause record is kept; the done date gets its own period.
    assert_eq!(periods[0]["ended_at"], paused_end);
    assert_eq!(periods[1]["started_at"], periods[1]["ended_at"]);
    assert_eq!(done["completed_at"], periods[1]["ended_at"]);
}

#[sqlx::test]
async fn archiving_an_active_node_closes_its_period(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "archiveperiod@example.com").await;
    let node_id = project(&app, &token).await;

    patch_status(&app, &token, &node_id, "active", true).await;
    let archived = patch_status(&app, &token, &node_id, "archived", true).await;
    assert_eq!(archived["completed_at"], Value::Null);
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 1);
    assert!(!periods[0]["ended_at"].is_null());
}

#[sqlx::test]
async fn never_started_node_going_done_gets_no_dates(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "neverstarted@example.com").await;
    let node_id = project(&app, &token).await;

    patch_status(&app, &token, &node_id, "queued", true).await;
    let done = patch_status(&app, &token, &node_id, "done", true).await;
    assert_eq!(done["started_at"], Value::Null);
    assert_eq!(done["completed_at"], Value::Null);
    assert!(list_periods(&app, &token, &node_id).await.is_empty());

    // And a completion date can't be set by hand either.
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{node_id}"),
            json!({"completed_at": "2026-02-10T00:00:00Z"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn deleting_the_first_period_makes_started_follow_the_next(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "deletefirst@example.com").await;
    let node_id = project(&app, &token).await;

    patch_status(&app, &token, &node_id, "active", true).await;
    patch_status(&app, &token, &node_id, "paused", true).await;
    patch_status(&app, &token, &node_id, "active", true).await;
    let periods = list_periods(&app, &token, &node_id).await;
    assert_eq!(periods.len(), 2);

    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/periods/{}", periods[0]["id"].as_str().unwrap()),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, node) = send(
        &app,
        req(
            "GET",
            &format!("/api/nodes/{node_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(node["started_at"], periods[1]["started_at"]);
}
