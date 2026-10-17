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

async fn create_node(app: &axum::Router, token: &str, kind: &str, title: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": kind, "title": title}),
            Some(token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    body["id"].as_str().unwrap().to_owned()
}

async fn create_note(app: &axum::Router, token: &str, body: Value) -> (StatusCode, Value) {
    send(app, req("POST", "/api/canvas/notes", body, Some(token))).await
}

async fn canvas(app: &axum::Router, token: &str) -> Value {
    let (status, body) = send(
        app,
        req("GET", "/api/board/canvas", Value::Null, Some(token)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body
}

#[sqlx::test]
async fn note_crud_round_trip(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "notes@example.com").await;

    let (status, note) = create_note(
        &app,
        &token,
        json!({"body": "remember this", "canvas_x": 10.0, "canvas_y": 20.0}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(note["canvas_width"], 200.0);
    assert_eq!(note["path_id"], Value::Null);
    let note_id = note["id"].as_str().unwrap();

    let (status, updated) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/canvas/notes/{note_id}"),
            json!({"body": "changed", "canvas_width": 300.0}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["body"], "changed");
    assert_eq!(updated["canvas_width"], 300.0);
    assert_eq!(updated["canvas_x"], 10.0);

    assert_eq!(
        canvas(&app, &token).await["notes"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/canvas/notes/{note_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(
        canvas(&app, &token).await["notes"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn note_rejects_tiny_size_and_non_path_parent(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "notes@example.com").await;
    let project = create_node(&app, &token, "project", "Project").await;

    let (status, _) = create_note(
        &app,
        &token,
        json!({"canvas_x": 0.0, "canvas_y": 0.0, "canvas_width": 10.0}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = create_note(
        &app,
        &token,
        json!({"canvas_x": 0.0, "canvas_y": 0.0, "path_id": project}),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn notes_and_arrows_are_scoped_to_their_owner(pool: PgPool) {
    let app = app(pool);
    let owner = signup(&app, "owner@example.com").await;
    let intruder = signup(&app, "intruder@example.com").await;
    let path = create_node(&app, &owner, "path", "Path").await;
    let node = create_node(&app, &owner, "project", "Project").await;
    let (_, note) = create_note(&app, &owner, json!({"canvas_x": 0.0, "canvas_y": 0.0})).await;
    let note_id = note["id"].as_str().unwrap();
    let (status, arrow) = send(
        &app,
        req(
            "POST",
            "/api/canvas/arrows",
            json!({"from_note_id": note_id, "to_node_id": node}),
            Some(&owner),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let arrow_id = arrow["id"].as_str().unwrap();

    // Can't see, edit or delete someone else's notes and arrows.
    let other_canvas = canvas(&app, &intruder).await;
    assert!(other_canvas["notes"].as_array().unwrap().is_empty());
    assert!(other_canvas["arrows"].as_array().unwrap().is_empty());
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/canvas/notes/{note_id}"),
            json!({"body": "hijacked"}),
            Some(&intruder),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/canvas/notes/{note_id}"),
            Value::Null,
            Some(&intruder),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/canvas/arrows/{arrow_id}"),
            Value::Null,
            Some(&intruder),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Can't put a note in someone else's path or point at their items.
    let (status, _) = create_note(
        &app,
        &intruder,
        json!({"canvas_x": 0.0, "canvas_y": 0.0, "path_id": path}),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, own_note) =
        create_note(&app, &intruder, json!({"canvas_x": 0.0, "canvas_y": 0.0})).await;
    for body in [
        json!({"from_note_id": own_note["id"], "to_node_id": node}),
        json!({"from_note_id": own_note["id"], "to_note_id": note_id}),
    ] {
        let (status, _) = send(
            &app,
            req("POST", "/api/canvas/arrows", body, Some(&intruder)),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}

#[sqlx::test]
async fn arrow_needs_one_distinct_item_per_end(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "arrows@example.com").await;
    let node = create_node(&app, &token, "project", "Project").await;
    let (_, note) = create_note(&app, &token, json!({"canvas_x": 0.0, "canvas_y": 0.0})).await;
    for body in [
        json!({"from_node_id": node}),
        json!({"from_node_id": node, "from_note_id": note["id"], "to_node_id": node}),
        json!({"from_node_id": node, "to_node_id": node}),
    ] {
        let (status, _) = send(&app, req("POST", "/api/canvas/arrows", body, Some(&token))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

#[sqlx::test]
async fn deleting_an_end_removes_its_arrows(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "arrows@example.com").await;
    let from = create_node(&app, &token, "project", "From").await;
    let to = create_node(&app, &token, "project", "To").await;
    let (status, arrow) = send(
        &app,
        req(
            "POST",
            "/api/canvas/arrows",
            json!({"from_node_id": from, "to_node_id": to}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        (arrow["from_side"].as_str(), arrow["to_side"].as_str()),
        (Some("right"), Some("left"))
    );
    let (status, arrow) = send(
        &app,
        req(
            "POST",
            "/api/canvas/arrows",
            json!({"from_node_id": to, "to_node_id": from, "from_side": "top", "to_side": "bottom"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        (arrow["from_side"].as_str(), arrow["to_side"].as_str()),
        (Some("top"), Some("bottom"))
    );
    let (status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/nodes/{to}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(
        canvas(&app, &token).await["arrows"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn notes_leave_a_path_that_stops_being_one(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "notes@example.com").await;
    let path = create_node(&app, &token, "path", "Path").await;
    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{path}"),
            json!({"canvas_x": 100.0, "canvas_y": 50.0}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, note) = create_note(
        &app,
        &token,
        json!({"canvas_x": 30.0, "canvas_y": 60.0, "path_id": path}),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(note["path_id"], path.as_str());

    let (status, _) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/nodes/{path}"),
            json!({"kind": "project"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let notes = canvas(&app, &token).await["notes"].clone();
    assert_eq!(notes[0]["path_id"], Value::Null);
    // Same spot on screen: offset by the old path's position.
    assert_eq!(notes[0]["canvas_x"], 130.0);
    assert_eq!(notes[0]["canvas_y"], 110.0);
}
