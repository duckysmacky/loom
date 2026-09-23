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

#[sqlx::test]
async fn create_get_update_delete_happy_path(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "topiccrud@example.com").await;

    let (status, topic) = send(
        &app,
        req(
            "POST",
            "/api/topics",
            json!({"name": "rust", "color": "#dea584"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let topic_id = topic["id"].as_str().unwrap().to_owned();

    let (_, list) = send(&app, req("GET", "/api/topics", Value::Null, Some(&token))).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    let (_, updated) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/topics/{topic_id}"),
            json!({"name": "rust-lang"}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(updated["name"], "rust-lang");
    assert_eq!(updated["color"], "#dea584"); // untouched field kept

    let (_, cleared) = send(
        &app,
        req(
            "PATCH",
            &format!("/api/topics/{topic_id}"),
            json!({"color": null}),
            Some(&token),
        ),
    )
    .await;
    assert_eq!(cleared["color"], Value::Null);

    let (delete_status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/topics/{topic_id}"),
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
            &format!("/api/topics/{topic_id}"),
            Value::Null,
            Some(&token),
        ),
    )
    .await;
    assert_eq!(get_after_delete, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn duplicate_topic_name_returns_409(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "dup@example.com").await;

    let (first, _) = send(
        &app,
        req("POST", "/api/topics", json!({"name": "rust"}), Some(&token)),
    )
    .await;
    assert_eq!(first, StatusCode::CREATED);

    let (second, _) = send(
        &app,
        req("POST", "/api/topics", json!({"name": "rust"}), Some(&token)),
    )
    .await;
    assert_eq!(second, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn ownership_scoping_returns_404_for_another_users_topic(pool: PgPool) {
    let app = app(pool);
    let token_a = signup(&app, "topicowner2@example.com").await;
    let token_b = signup(&app, "topicintruder2@example.com").await;

    let (_, topic) = send(
        &app,
        req(
            "POST",
            "/api/topics",
            json!({"name": "mine"}),
            Some(&token_a),
        ),
    )
    .await;
    let topic_id = topic["id"].as_str().unwrap().to_owned();

    let (get_status, _) = send(
        &app,
        req(
            "GET",
            &format!("/api/topics/{topic_id}"),
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
            &format!("/api/topics/{topic_id}"),
            json!({"name": "hijacked"}),
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(patch_status, StatusCode::NOT_FOUND);

    let (delete_status, _) = send(
        &app,
        req(
            "DELETE",
            &format!("/api/topics/{topic_id}"),
            Value::Null,
            Some(&token_b),
        ),
    )
    .await;
    assert_eq!(delete_status, StatusCode::NOT_FOUND);

    let (_, list_b) = send(&app, req("GET", "/api/topics", Value::Null, Some(&token_b))).await;
    assert_eq!(list_b.as_array().unwrap().len(), 0);
}
