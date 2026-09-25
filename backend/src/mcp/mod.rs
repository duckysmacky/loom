//! The optional MCP server (`MCP_ENABLED=true`): a stateless Streamable
//! HTTP endpoint at `/mcp` that lets AI agents drive the graph through the
//! same handlers the web app uses.

mod tools;

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use serde_json::json;

use crate::auth::refresh_token::hash_token;
use crate::handlers::error::ApiError;
use crate::handlers::oauth::protected_resource_metadata_url;
use crate::middleware::auth_user::AuthUser;
use crate::repo::mcp_tokens;
use crate::state::AppState;

pub use tools::LoomServer;

/// The `/mcp` route, behind bearer-token auth. `public_url` is the origin
/// agents reach it at - its host is the only non-loopback `Host` accepted.
pub fn router(state: AppState, public_url: &str) -> Router {
    let public_host = public_url
        .parse::<Uri>()
        .ok()
        .and_then(|uri| uri.host().map(str::to_string))
        .unwrap_or_default();
    // Stateless: every POST is served on its own, so there's no session
    // map to leak and nothing lost on restart. Plain JSON responses keep
    // it friendly to buffering proxies.
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_allowed_hosts([public_host, "localhost".into(), "127.0.0.1".into()]);

    let tool_state = state.clone();
    let service = StreamableHttpService::new(
        move || Ok(LoomServer::new(tool_state.clone())),
        Arc::new(NeverSessionManager::default()),
        config,
    );

    Router::new()
        .route_service("/mcp", service)
        .route_layer(middleware::from_fn_with_state(state, authenticate))
}

/// Resolves the bearer token (a personal token or an OAuth access token,
/// both from `mcp_tokens` - never a web-session JWT) to the caller and
/// hands it to the tools as an `AuthUser` request extension.
async fn authenticate(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
        .map(|(_, token)| token.trim());

    let user_id = match token {
        Some(token) => mcp_tokens::authenticate(&state.pool, &hash_token(token)).await,
        None => Ok(None),
    };
    match user_id {
        Ok(Some(user_id)) => {
            request.extensions_mut().insert(AuthUser { user_id });
            next.run(request).await
        }
        Ok(None) => unauthorized(state.mcp_public_url.as_deref().unwrap_or_default()),
        Err(error) => ApiError::from(error).into_response(),
    }
}

/// The 401 points OAuth-capable clients at the resource metadata, which
/// is how they discover where to sign in (MCP authorization spec).
fn unauthorized(public_url: &str) -> Response {
    let challenge = format!(
        r#"Bearer resource_metadata="{}""#,
        protected_resource_metadata_url(public_url)
    );
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, challenge)],
        Json(json!({ "error": "invalid or missing token" })),
    )
        .into_response()
}
