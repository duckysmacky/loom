use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use jsonwebtoken::{EncodingKey, Header, encode};
use loom::app::build_router;
use loom::auth::jwt::Claims;
use loom::state::AppState;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "test-only-secret";

fn app(pool: PgPool) -> axum::Router {
    build_router(AppState::new(pool, TEST_JWT_SECRET))
}

async fn send(app: &axum::Router, req: Request<Body>) -> (StatusCode, Value, Option<String>) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let cookie = response
        .headers()
        .get(header::SET_COOKIE)
        .map(|v| v.to_str().unwrap().to_owned());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    // Non-JSON bodies (e.g. the rate limiter's own 429 response) just become
    // Null - only the JSON-returning success/error paths assert on body shape.
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body, cookie)
}

const TEST_PEER: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    12345,
);

/// Every request carries a `ConnectInfo`, same as `into_make_service_with_connect_info`
/// would attach in production - the /login rate limiter's key extractor needs
/// it on every request through this router, not just the ones testing it.
fn json_request(method: &str, uri: &str, body: Value, cookie: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .extension(ConnectInfo(TEST_PEER));
    if let Some(cookie) = cookie {
        builder = builder.header(header::COOKIE, cookie);
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

/// The raw `name=value` pair from a Set-Cookie header, suitable for
/// resending as a Cookie header - there's no cookie jar under `oneshot`.
fn cookie_pair(set_cookie: &str) -> &str {
    set_cookie.split(';').next().unwrap()
}

#[sqlx::test]
async fn signup_creates_user_and_returns_tokens(pool: PgPool) {
    let app = app(pool);
    let (status, body, cookie) = send(
        &app,
        json_request(
            "POST",
            "/api/auth/signup",
            json!({"email": "signup@example.com", "password": "password123"}),
            None,
        ),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["user"]["email"], "signup@example.com");
    assert!(!body["access_token"].as_str().unwrap().is_empty());
    assert!(cookie.unwrap().contains("HttpOnly"));
}

#[sqlx::test]
async fn signup_rejects_malformed_email(pool: PgPool) {
    let app = app(pool);

    for email in ["noatsign.example.com", "a@b", "has space@example.com"] {
        let (status, ..) = send(
            &app,
            json_request(
                "POST",
                "/api/auth/signup",
                json!({"email": email, "password": "password123"}),
                None,
            ),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "email {email:?} should be rejected"
        );
    }
}

#[sqlx::test]
async fn signup_duplicate_email_returns_409(pool: PgPool) {
    let app = app(pool);
    let body = json!({"email": "dup@example.com", "password": "password123"});

    let (first, ..) = send(
        &app,
        json_request("POST", "/api/auth/signup", body.clone(), None),
    )
    .await;
    assert_eq!(first, StatusCode::CREATED);

    let (second, ..) = send(&app, json_request("POST", "/api/auth/signup", body, None)).await;
    assert_eq!(second, StatusCode::CONFLICT);
}

#[sqlx::test]
async fn login_success_returns_tokens(pool: PgPool) {
    let app = app(pool);
    let body = json!({"email": "login@example.com", "password": "password123"});
    send(
        &app,
        json_request("POST", "/api/auth/signup", body.clone(), None),
    )
    .await;

    let (status, response_body, _) =
        send(&app, json_request("POST", "/api/auth/login", body, None)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response_body["user"]["email"], "login@example.com");
}

#[sqlx::test]
async fn login_wrong_password_and_unknown_email_both_return_401(pool: PgPool) {
    let app = app(pool);
    send(
        &app,
        json_request(
            "POST",
            "/api/auth/signup",
            json!({"email": "wrongpw@example.com", "password": "password123"}),
            None,
        ),
    )
    .await;

    let (wrong_pw_status, wrong_pw_body, _) = send(
        &app,
        json_request(
            "POST",
            "/api/auth/login",
            json!({"email": "wrongpw@example.com", "password": "not-the-password"}),
            None,
        ),
    )
    .await;
    let (unknown_status, unknown_body, _) = send(
        &app,
        json_request(
            "POST",
            "/api/auth/login",
            json!({"email": "nobody@example.com", "password": "whatever12"}),
            None,
        ),
    )
    .await;

    assert_eq!(wrong_pw_status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown_status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_pw_body, unknown_body); // no user enumeration
}

#[sqlx::test]
async fn refresh_rotates_token_and_old_cookie_rejected(pool: PgPool) {
    let app = app(pool);
    let (_, _, signup_cookie) = send(
        &app,
        json_request(
            "POST",
            "/api/auth/signup",
            json!({"email": "refresh@example.com", "password": "password123"}),
            None,
        ),
    )
    .await;
    let old_cookie = cookie_pair(&signup_cookie.unwrap()).to_owned();

    let (refresh_status, refresh_body, rotated_cookie) = send(
        &app,
        json_request("POST", "/api/auth/refresh", Value::Null, Some(&old_cookie)),
    )
    .await;
    assert_eq!(refresh_status, StatusCode::OK);
    assert!(!refresh_body["access_token"].as_str().unwrap().is_empty());
    let new_cookie = cookie_pair(&rotated_cookie.unwrap()).to_owned();
    assert_ne!(old_cookie, new_cookie);

    // Replaying the old (now-rotated-away) cookie must be rejected.
    let (replay_status, ..) = send(
        &app,
        json_request("POST", "/api/auth/refresh", Value::Null, Some(&old_cookie)),
    )
    .await;
    assert_eq!(replay_status, StatusCode::UNAUTHORIZED);

    // The new cookie still works.
    let (still_works_status, ..) = send(
        &app,
        json_request("POST", "/api/auth/refresh", Value::Null, Some(&new_cookie)),
    )
    .await;
    assert_eq!(still_works_status, StatusCode::OK);
}

#[sqlx::test]
async fn logout_revokes_refresh_token(pool: PgPool) {
    let app = app(pool);
    let (_, _, signup_cookie) = send(
        &app,
        json_request(
            "POST",
            "/api/auth/signup",
            json!({"email": "logout@example.com", "password": "password123"}),
            None,
        ),
    )
    .await;
    let cookie = cookie_pair(&signup_cookie.unwrap()).to_owned();

    let (logout_status, ..) = send(
        &app,
        json_request("POST", "/api/auth/logout", Value::Null, Some(&cookie)),
    )
    .await;
    assert_eq!(logout_status, StatusCode::NO_CONTENT);

    let (refresh_status, ..) = send(
        &app,
        json_request("POST", "/api/auth/refresh", Value::Null, Some(&cookie)),
    )
    .await;
    assert_eq!(refresh_status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn me_rejects_missing_garbage_and_expired_bearer(pool: PgPool) {
    let app = app(pool);

    let missing = Request::builder()
        .uri("/api/auth/me")
        .body(Body::empty())
        .unwrap();
    let (missing_status, ..) = send(&app, missing).await;
    assert_eq!(missing_status, StatusCode::UNAUTHORIZED);

    let garbage = Request::builder()
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, "Bearer not-a-real-token")
        .body(Body::empty())
        .unwrap();
    let (garbage_status, ..) = send(&app, garbage).await;
    assert_eq!(garbage_status, StatusCode::UNAUTHORIZED);

    let expired_claims = Claims {
        sub: Uuid::new_v4(),
        iat: 0,
        exp: 1, // long in the past
    };
    let expired_token = encode(
        &Header::default(),
        &expired_claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .unwrap();
    let expired = Request::builder()
        .uri("/api/auth/me")
        .header(header::AUTHORIZATION, format!("Bearer {expired_token}"))
        .body(Body::empty())
        .unwrap();
    let (expired_status, ..) = send(&app, expired).await;
    assert_eq!(expired_status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn login_rate_limited_after_repeated_failures(pool: PgPool) {
    let app = app(pool);

    let mut last_status = StatusCode::OK;
    for _ in 0..10 {
        let req = json_request(
            "POST",
            "/api/auth/login",
            json!({"email": "nope@example.com", "password": "wrong-password"}),
            None,
        );
        let (status, ..) = send(&app, req).await;
        last_status = status;
    }

    assert_eq!(last_status, StatusCode::TOO_MANY_REQUESTS);
}
