use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http_body_util::BodyExt;
use loom::app::build_router;
use loom::state::AppState;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tower::ServiceExt;

const TEST_JWT_SECRET: &str = "test-only-secret";
const PUBLIC_URL: &str = "https://loom.example.com";
const REDIRECT_URI: &str = "https://claude.ai/api/mcp/auth_callback";
const VERIFIER: &str = "a-very-long-and-random-pkce-code-verifier-0123456789";

const TEST_PEER: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    12345,
);

fn app(pool: PgPool) -> axum::Router {
    build_router(AppState::new(pool, TEST_JWT_SECRET).with_mcp(PUBLIC_URL))
}

async fn send(app: &axum::Router, req: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

fn builder(method: &str, uri: &str, token: Option<&str>) -> axum::http::request::Builder {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::HOST, "loom.example.com")
        .header(header::ACCEPT, "application/json, text/event-stream")
        .extension(ConnectInfo(TEST_PEER));
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    builder
}

fn req(method: &str, uri: &str, body: Value, token: Option<&str>) -> Request<Body> {
    builder(method, uri, token)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn token_request(app: &axum::Router, form: &[(&str, &str)]) -> (StatusCode, Value) {
    let body = serde_urlencoded::to_string(form).unwrap();
    let request = builder("POST", "/api/oauth/token", None)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .unwrap();
    send(app, request).await
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

async fn register(app: &axum::Router) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/oauth/register",
            json!({"client_name": "Claude", "redirect_uris": [REDIRECT_URI]}),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["client_id"].as_str().unwrap().to_owned()
}

fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn authorize_body(client_id: &str, approve: bool) -> Value {
    json!({
        "response_type": "code",
        "client_id": client_id,
        "redirect_uri": REDIRECT_URI,
        "code_challenge": challenge(VERIFIER),
        "code_challenge_method": "S256",
        "state": "xyz",
        "resource": format!("{PUBLIC_URL}/mcp"),
        "approve": approve,
    })
}

/// Approves the client as the user and returns the authorization code.
async fn approve(app: &axum::Router, jwt: &str, client_id: &str) -> String {
    let (status, body) = send(
        app,
        req(
            "POST",
            "/api/oauth/authorize",
            authorize_body(client_id, true),
            Some(jwt),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let redirect_to = body["redirect_to"].as_str().unwrap();
    let query = redirect_to
        .strip_prefix(&format!("{REDIRECT_URI}?"))
        .unwrap();
    let params: Vec<(String, String)> = serde_urlencoded::from_str(query).unwrap();
    let get = |key: &str| {
        params
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    assert_eq!(get("state").as_deref(), Some("xyz"));
    assert_eq!(get("iss").as_deref(), Some(PUBLIC_URL));
    get("code").unwrap()
}

async fn exchange(
    app: &axum::Router,
    client_id: &str,
    code: &str,
    verifier: &str,
) -> (StatusCode, Value) {
    token_request(
        app,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", REDIRECT_URI),
            ("client_id", client_id),
            ("code_verifier", verifier),
            ("resource", &format!("{PUBLIC_URL}/mcp")),
        ],
    )
    .await
}

async fn mcp_status(app: &axum::Router, access_token: &str) -> StatusCode {
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}});
    send(app, req("POST", "/mcp", body, Some(access_token)))
        .await
        .0
}

/// Runs the whole flow for a fresh user; returns (web JWT, client_id, token response).
async fn connected_user(app: &axum::Router, email: &str) -> (String, String, Value) {
    let jwt = signup(app, email).await;
    let client_id = register(app).await;
    let code = approve(app, &jwt, &client_id).await;
    let (status, tokens) = exchange(app, &client_id, &code, VERIFIER).await;
    assert_eq!(status, StatusCode::OK, "{tokens}");
    (jwt, client_id, tokens)
}

#[sqlx::test]
async fn publishes_discovery_metadata(pool: PgPool) {
    let app = app(pool);
    let (status, resource) = send(
        &app,
        req(
            "GET",
            "/.well-known/oauth-protected-resource/mcp",
            Value::Null,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(resource["resource"], "https://loom.example.com/mcp");
    assert_eq!(resource["authorization_servers"], json!([PUBLIC_URL]));

    let (_, server) = send(
        &app,
        req(
            "GET",
            "/.well-known/oauth-authorization-server",
            Value::Null,
            None,
        ),
    )
    .await;
    assert_eq!(server["issuer"], PUBLIC_URL);
    assert_eq!(
        server["authorization_endpoint"],
        "https://loom.example.com/oauth/authorize"
    );
    assert_eq!(server["code_challenge_methods_supported"], json!(["S256"]));

    let response = app
        .clone()
        .oneshot(req("POST", "/mcp", json!({}), None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(
        response.headers()[header::WWW_AUTHENTICATE],
        r#"Bearer resource_metadata="https://loom.example.com/.well-known/oauth-protected-resource/mcp""#
    );
}

#[sqlx::test]
async fn oauth_routes_absent_when_mcp_disabled(pool: PgPool) {
    let app = build_router(AppState::new(pool, TEST_JWT_SECRET));
    let (status, _) = send(
        &app,
        req(
            "GET",
            "/.well-known/oauth-authorization-server",
            Value::Null,
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn register_rejects_unsafe_redirect_uris(pool: PgPool) {
    let app = app(pool);
    for uri in [
        "http://evil.example.com/cb",
        "javascript:alert(1)",
        "https://x.com/cb#frag",
    ] {
        let (status, body) = send(
            &app,
            req(
                "POST",
                "/api/oauth/register",
                json!({"redirect_uris": [uri]}),
                None,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(body["error"], "invalid_redirect_uri");
    }
}

#[sqlx::test]
async fn full_flow_grants_mcp_access(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "flow@example.com").await;
    let client_id = register(&app).await;

    let query = serde_urlencoded::to_string([
        ("response_type", "code"),
        ("client_id", client_id.as_str()),
        ("redirect_uri", REDIRECT_URI),
        ("code_challenge", challenge(VERIFIER).as_str()),
        ("code_challenge_method", "S256"),
    ])
    .unwrap();
    let uri = format!("/api/oauth/authorize?{query}");
    let (status, _) = send(&app, req("GET", &uri, Value::Null, None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "consent requires login");
    let (status, preview) = send(&app, req("GET", &uri, Value::Null, Some(&jwt))).await;
    assert_eq!(status, StatusCode::OK, "{preview}");
    assert_eq!(
        preview,
        json!({"client_name": "Claude", "redirect_host": "claude.ai"})
    );

    let code = approve(&app, &jwt, &client_id).await;
    let (status, tokens) = exchange(&app, &client_id, &code, VERIFIER).await;
    assert_eq!(status, StatusCode::OK, "{tokens}");
    assert_eq!(tokens["token_type"], "Bearer");
    assert_eq!(tokens["expires_in"], 3600);
    assert_eq!(
        mcp_status(&app, tokens["access_token"].as_str().unwrap()).await,
        StatusCode::OK
    );

    // A refresh token is not an access token.
    assert_eq!(
        mcp_status(&app, tokens["refresh_token"].as_str().unwrap()).await,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test]
async fn deny_redirects_with_access_denied(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "deny@example.com").await;
    let client_id = register(&app).await;
    let (status, body) = send(
        &app,
        req(
            "POST",
            "/api/oauth/authorize",
            authorize_body(&client_id, false),
            Some(&jwt),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let redirect_to = body["redirect_to"].as_str().unwrap();
    assert!(redirect_to.starts_with(&format!("{REDIRECT_URI}?error=access_denied&state=xyz")));
    assert!(!redirect_to.contains("code="));
}

#[sqlx::test]
async fn authorize_rejects_unregistered_redirect_and_missing_pkce(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "strict@example.com").await;
    let client_id = register(&app).await;

    let mut body = authorize_body(&client_id, true);
    body["redirect_uri"] = json!("https://evil.example.com/cb");
    let (status, _) = send(&app, req("POST", "/api/oauth/authorize", body, Some(&jwt))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let mut body = authorize_body(&client_id, true);
    body["code_challenge_method"] = json!("plain");
    let (status, _) = send(&app, req("POST", "/api/oauth/authorize", body, Some(&jwt))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let mut body = authorize_body(&client_id, true);
    body["resource"] = json!("https://other.example.com/mcp");
    let (status, _) = send(&app, req("POST", "/api/oauth/authorize", body, Some(&jwt))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn code_is_single_use_and_pkce_bound(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "code@example.com").await;
    let client_id = register(&app).await;

    let code = approve(&app, &jwt, &client_id).await;
    let (status, body) = exchange(
        &app,
        &client_id,
        &code,
        "the-wrong-verifier-but-long-enough-to-be-valid-000",
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_grant");
    // A failed attempt burns the code too.
    let (status, _) = exchange(&app, &client_id, &code, VERIFIER).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let code = approve(&app, &jwt, &client_id).await;
    let (status, _) = exchange(&app, &client_id, &code, VERIFIER).await;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = exchange(&app, &client_id, &code, VERIFIER).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_grant");
}

#[sqlx::test]
async fn code_is_bound_to_client_and_redirect_uri(pool: PgPool) {
    let app = app(pool);
    let jwt = signup(&app, "bound@example.com").await;
    let client_id = register(&app).await;
    let other_client = register(&app).await;

    let code = approve(&app, &jwt, &client_id).await;
    let (status, _) = exchange(&app, &other_client, &code, VERIFIER).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let code = approve(&app, &jwt, &client_id).await;
    let (status, _) = token_request(
        &app,
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", "https://claude.ai/other"),
            ("client_id", &client_id),
            ("code_verifier", VERIFIER),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn expired_code_is_rejected(pool: PgPool) {
    let app = app(pool.clone());
    let jwt = signup(&app, "expired@example.com").await;
    let client_id = register(&app).await;
    let code = approve(&app, &jwt, &client_id).await;
    sqlx::query("UPDATE oauth_codes SET expires_at = now() - interval '1 second'")
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = exchange(&app, &client_id, &code, VERIFIER).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_grant");
}

#[sqlx::test]
async fn refresh_token_rotates(pool: PgPool) {
    let app = app(pool);
    let (_, client_id, tokens) = connected_user(&app, "refresh@example.com").await;
    let old_refresh = tokens["refresh_token"].as_str().unwrap();

    let refresh = |token: String| {
        let app = app.clone();
        let client_id = client_id.clone();
        async move {
            token_request(
                &app,
                &[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", &token),
                    ("client_id", &client_id),
                ],
            )
            .await
        }
    };

    let (status, fresh) = refresh(old_refresh.to_owned()).await;
    assert_eq!(status, StatusCode::OK, "{fresh}");
    assert_ne!(fresh["refresh_token"], tokens["refresh_token"]);
    assert_eq!(
        mcp_status(&app, fresh["access_token"].as_str().unwrap()).await,
        StatusCode::OK
    );

    let (status, body) = refresh(old_refresh.to_owned()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "invalid_grant");

    let (status, _) = token_request(&app, &[("grant_type", "password")]).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn revoking_a_client_signs_it_out(pool: PgPool) {
    let app = app(pool);
    let (jwt, client_id, tokens) = connected_user(&app, "revoke@example.com").await;
    let access = tokens["access_token"].as_str().unwrap();

    let (status, clients) = send(
        &app,
        req("GET", "/api/oauth/clients", Value::Null, Some(&jwt)),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(clients.as_array().unwrap().len(), 1);
    assert_eq!(clients[0]["client_name"], "Claude");

    let uri = format!("/api/oauth/clients/{client_id}");
    let (status, _) = send(&app, req("DELETE", &uri, Value::Null, Some(&jwt))).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    assert_eq!(mcp_status(&app, access).await, StatusCode::UNAUTHORIZED);
    let (status, _) = token_request(
        &app,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", tokens["refresh_token"].as_str().unwrap()),
        ],
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[sqlx::test]
async fn cannot_see_or_revoke_another_users_clients(pool: PgPool) {
    let app = app(pool);
    let (_, client_id, tokens) = connected_user(&app, "owner@example.com").await;
    let intruder = signup(&app, "intruder@example.com").await;

    let (_, clients) = send(
        &app,
        req("GET", "/api/oauth/clients", Value::Null, Some(&intruder)),
    )
    .await;
    assert!(clients.as_array().unwrap().is_empty());

    let uri = format!("/api/oauth/clients/{client_id}");
    let (status, _) = send(&app, req("DELETE", &uri, Value::Null, Some(&intruder))).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        mcp_status(&app, tokens["access_token"].as_str().unwrap()).await,
        StatusCode::OK
    );
}

#[sqlx::test]
async fn oauth_tokens_act_as_the_approving_user(pool: PgPool) {
    let app = app(pool);
    let (owner_jwt, _, _) = connected_user(&app, "a@example.com").await;
    let (_, _, other_tokens) = connected_user(&app, "b@example.com").await;

    let (_, node) = send(
        &app,
        req(
            "POST",
            "/api/nodes",
            json!({"kind": "idea", "title": "Private"}),
            Some(&owner_jwt),
        ),
    )
    .await;
    let body = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "loom_get_node", "arguments": {"node_id": node["id"]}}});
    let (status, response) = send(
        &app,
        req("POST", "/mcp", body, other_tokens["access_token"].as_str()),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(response["result"]["isError"], json!(true));
}
