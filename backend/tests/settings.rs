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

async fn settings(app: &axum::Router, token: &str) -> Value {
    let (status, body) = send(app, req("GET", "/api/settings", Value::Null, Some(token))).await;
    assert_eq!(status, StatusCode::OK);
    normalize_whole_number_floats(body)
}

fn normalize_whole_number_floats(value: Value) -> Value {
    match value {
        Value::Number(number) => {
            if let Some(float) = number.as_f64() {
                if float.fract() == 0.0
                    && float.is_finite()
                    && float >= i64::MIN as f64
                    && float <= i64::MAX as f64
                {
                    return Value::Number((float as i64).into());
                }
            }
            Value::Number(number)
        }
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(normalize_whole_number_floats)
                .collect(),
        ),
        Value::Object(entries) => Value::Object(
            entries
                .into_iter()
                .map(|(key, value)| (key, normalize_whole_number_floats(value)))
                .collect(),
        ),
        other => other,
    }
}

#[sqlx::test]
async fn get_returns_defaults_for_a_new_account(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "defaults@example.com").await;
    let body = settings(&app, &token).await;
    assert_eq!(body["theme"], "system");
    assert_eq!(body["stale_days"], 14);
    assert_eq!(body["track_active_periods"], true);
}

#[sqlx::test]
async fn patch_merges_only_the_sent_keys(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "merge@example.com").await;
    for changes in [json!({"theme": "dark"}), json!({"density": "compact"})] {
        let (status, _) = send(&app, req("PATCH", "/api/settings", changes, Some(&token))).await;
        assert_eq!(status, StatusCode::OK);
    }
    let body = settings(&app, &token).await;
    assert_eq!(body["theme"], "dark");
    assert_eq!(body["density"], "compact");
    assert_eq!(body["stale_days"], 14);
}

#[sqlx::test]
async fn patch_rejects_bad_input_without_storing_anything(pool: PgPool) {
    let app = app(pool);
    let token = signup(&app, "invalid@example.com").await;
    for changes in [
        json!({"nope": true}),
        json!({"theme": "neon"}),
        json!({"stale_days": 0}),
        json!({"stale_days": 366}),
        json!({"stale_days": "14"}),
        json!({"recent_colors": ["red"]}),
        json!({"recent_colors": vec!["#fff"; 9]}),
        // A valid key does not rescue an invalid sibling.
        json!({"theme": "dark", "density": "huge"}),
    ] {
        let (status, _) = send(
            &app,
            req("PATCH", "/api/settings", changes.clone(), Some(&token)),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{changes}");
    }
    assert_eq!(settings(&app, &token).await["theme"], "system");
}

#[sqlx::test]
async fn settings_are_scoped_to_the_account(pool: PgPool) {
    let app = app(pool);
    let alice = signup(&app, "alice@example.com").await;
    let bob = signup(&app, "bob@example.com").await;
    send(
        &app,
        req(
            "PATCH",
            "/api/settings",
            json!({"stale_days": 30}),
            Some(&alice),
        ),
    )
    .await;
    assert_eq!(settings(&app, &alice).await["stale_days"], 30);
    assert_eq!(settings(&app, &bob).await["stale_days"], 14);
}

#[sqlx::test]
async fn settings_need_a_web_session(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "session@example.com").await;
    let (status, _) = send(&app, req("GET", "/api/settings", Value::Null, None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (_, created) = send(
        &app,
        req(
            "POST",
            "/api/tokens",
            json!({"name": "t", "scope": "read_write"}),
            Some(&jwt),
        ),
    )
    .await;
    let personal = created["raw_token"].as_str().unwrap();
    for method in ["GET", "PATCH"] {
        let (status, _) = send(
            &app,
            req(
                method,
                "/api/settings",
                json!({"theme": "dark"}),
                Some(personal),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{method}");
    }
}
