//! A minimal OAuth 2.1 authorization server for MCP connector clients -
//! just enough of RFC 8414 (metadata), RFC 9728 (protected resource
//! metadata), RFC 7591 (dynamic registration) and PKCE for Claude and
//! other MCP clients to connect with nothing but the server URL.
//! Only mounted when MCP is enabled.

use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath, ApiQuery};
use crate::auth::refresh_token::{self, REFRESH_TOKEN_TTL};
use crate::middleware::auth_user::AuthUser;
use crate::models::oauth::{
    AuthorizeDecision, AuthorizePreview, AuthorizeRedirect, AuthorizeRequest,
    ConnectedClientResponse, RegisterClientRequest, TokenRequest,
};
use crate::repo::mcp_tokens::{self, IssuedTokenHashes};
use crate::repo::oauth::{self, OAuthClient};
use crate::state::AppState;

const ACCESS_TOKEN_TTL: Duration = Duration::hours(1);
const CODE_TTL: Duration = Duration::minutes(10);
const MAX_REDIRECT_URIS: usize = 10;
const MAX_REDIRECT_URI_LEN: usize = 2000;
const MAX_CLIENT_NAME_LEN: usize = 100;

fn public_url(state: &AppState) -> &str {
    state
        .mcp_public_url
        .as_deref()
        .expect("OAuth routes are only mounted when MCP is enabled")
}

/// The canonical resource URI - what OAuth tokens here are issued for.
pub fn mcp_resource(public_url: &str) -> String {
    format!("{public_url}/mcp")
}

pub fn protected_resource_metadata_url(public_url: &str) -> String {
    format!("{public_url}/.well-known/oauth-protected-resource/mcp")
}

/// RFC 9728 - served at both the root and the `/mcp`-suffixed well-known
/// path, since clients try either.
pub async fn protected_resource_metadata(State(state): State<AppState>) -> Json<Value> {
    let public_url = public_url(&state);
    Json(json!({
        "resource": mcp_resource(public_url),
        "authorization_servers": [public_url],
        "bearer_methods_supported": ["header"],
        "resource_name": "Loom",
    }))
}

/// RFC 8414.
pub async fn authorization_server_metadata(State(state): State<AppState>) -> Json<Value> {
    let public_url = public_url(&state);
    Json(json!({
        "issuer": public_url,
        "authorization_endpoint": format!("{public_url}/oauth/authorize"),
        "token_endpoint": format!("{public_url}/api/oauth/token"),
        "registration_endpoint": format!("{public_url}/api/oauth/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"],
        "authorization_response_iss_parameter_supported": true,
    }))
}

/// RFC 7591 dynamic registration, public clients only (no secret - PKCE
/// protects the code instead).
pub async fn register(
    State(state): State<AppState>,
    ApiJson(request): ApiJson<RegisterClientRequest>,
) -> Result<(StatusCode, Json<Value>), OAuthError> {
    if request.redirect_uris.is_empty() || request.redirect_uris.len() > MAX_REDIRECT_URIS {
        return Err(OAuthError::bad_request(
            "invalid_redirect_uri",
            "provide between 1 and 10 redirect_uris",
        ));
    }
    if let Some(invalid) = request
        .redirect_uris
        .iter()
        .find(|uri| !is_allowed_redirect_uri(uri))
    {
        return Err(OAuthError::bad_request(
            "invalid_redirect_uri",
            format!("{invalid} must be https, or http on localhost, with no fragment"),
        ));
    }
    let client_name = request
        .client_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("MCP client")
        .chars()
        .take(MAX_CLIENT_NAME_LEN)
        .collect::<String>();

    let client_id = Uuid::new_v4().to_string();
    oauth::create_client(
        &state.pool,
        &client_id,
        &client_name,
        &request.redirect_uris,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "client_id": client_id,
            "client_id_issued_at": Utc::now().timestamp(),
            "client_name": client_name,
            "redirect_uris": request.redirect_uris,
            "token_endpoint_auth_method": "none",
            "grant_types": ["authorization_code", "refresh_token"],
            "response_types": ["code"],
        })),
    ))
}

/// https anywhere, plain http only to loopback (RFC 8252 native apps).
/// Anything else - notably `javascript:`/`data:` - is refused, since the
/// consent page navigates to this URI.
fn is_allowed_redirect_uri(uri: &str) -> bool {
    if uri.len() > MAX_REDIRECT_URI_LEN || uri.contains('#') {
        return false;
    }
    let Ok(parsed) = uri.parse::<Uri>() else {
        return false;
    };
    match (parsed.scheme_str(), parsed.host()) {
        (Some("https"), Some(host)) => !host.is_empty(),
        (Some("http"), Some(host)) => matches!(host, "localhost" | "127.0.0.1" | "[::1]"),
        _ => false,
    }
}

/// Checks an authorization request against its registered client. Errors
/// are shown on the consent page and never redirected: a request that
/// fails here may not have a trustworthy redirect URI.
async fn validate_authorize(
    state: &AppState,
    request: &AuthorizeRequest,
) -> Result<OAuthClient, ApiError> {
    let client = oauth::find_client(&state.pool, &request.client_id)
        .await?
        .ok_or(ApiError::InvalidInput(
            "unknown client - remove the connector and add it again",
        ))?;
    if !client.redirect_uris.contains(&request.redirect_uri) {
        return Err(ApiError::InvalidInput(
            "redirect_uri is not registered for this client",
        ));
    }
    if request.response_type != "code" {
        return Err(ApiError::InvalidInput("response_type must be code"));
    }
    if request.code_challenge_method != "S256" || request.code_challenge.len() != 43 {
        return Err(ApiError::InvalidInput("PKCE with S256 is required"));
    }
    if let Some(resource) = &request.resource
        && !is_this_resource(public_url(state), resource)
    {
        return Err(ApiError::InvalidInput(
            "resource is not this Loom server's MCP endpoint",
        ));
    }
    Ok(client)
}

fn is_this_resource(public_url: &str, resource: &str) -> bool {
    let resource = resource.trim_end_matches('/');
    resource.eq_ignore_ascii_case(&mcp_resource(public_url))
        || resource.eq_ignore_ascii_case(public_url)
}

/// `GET /api/oauth/authorize` - the consent page's first call: is this a
/// valid request, and who is asking?
pub async fn preview(
    State(state): State<AppState>,
    _user: AuthUser,
    ApiQuery(request): ApiQuery<AuthorizeRequest>,
) -> Result<Json<AuthorizePreview>, ApiError> {
    let client = validate_authorize(&state, &request).await?;
    let redirect_host = request
        .redirect_uri
        .parse::<Uri>()
        .ok()
        .and_then(|uri| uri.host().map(str::to_string))
        .unwrap_or_default();
    Ok(Json(AuthorizePreview {
        client_name: client.client_name,
        redirect_host,
    }))
}

/// `POST /api/oauth/authorize` - the user's approve/deny. Approval mints a
/// single-use code bound to the client, redirect URI and PKCE challenge.
pub async fn decide(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(decision): ApiJson<AuthorizeDecision>,
) -> Result<Json<AuthorizeRedirect>, ApiError> {
    let request = decision.request;
    validate_authorize(&state, &request).await?;
    let issuer = public_url(&state);

    let code = if decision.approve {
        let (raw, hash) = refresh_token::generate();
        oauth::create_code(
            user_id,
            &state.pool,
            &hash,
            &request.client_id,
            &request.redirect_uri,
            &request.code_challenge,
            Utc::now() + CODE_TTL,
        )
        .await?;
        Some(raw)
    } else {
        None
    };
    let mut params = match &code {
        Some(code) => vec![("code", code.as_str())],
        None => vec![("error", "access_denied")],
    };
    if let Some(client_state) = &request.state {
        params.push(("state", client_state));
    }
    params.push(("iss", issuer));

    let query =
        serde_urlencoded::to_string(&params).map_err(|error| ApiError::Internal(error.into()))?;
    let separator = if request.redirect_uri.contains('?') {
        '&'
    } else {
        '?'
    };
    Ok(Json(AuthorizeRedirect {
        redirect_to: format!("{}{separator}{query}", request.redirect_uri),
    }))
}

/// `POST /api/oauth/token` - exchanges a code (with its PKCE verifier) or a
/// refresh token for a new access/refresh pair.
pub async fn token(
    State(state): State<AppState>,
    form: Result<Form<TokenRequest>, FormRejection>,
) -> Result<Response, OAuthError> {
    let Form(request) = form
        .map_err(|rejection| OAuthError::bad_request("invalid_request", rejection.body_text()))?;
    if let Some(resource) = &request.resource
        && !is_this_resource(public_url(&state), resource)
    {
        return Err(OAuthError::bad_request(
            "invalid_target",
            "resource is not this Loom server's MCP endpoint",
        ));
    }

    let (user_id, client_id) = match request.grant_type.as_str() {
        "authorization_code" => exchange_code(&state, request).await?,
        "refresh_token" => exchange_refresh_token(&state, request).await?,
        _ => {
            return Err(OAuthError::bad_request(
                "unsupported_grant_type",
                "grant_type must be authorization_code or refresh_token",
            ));
        }
    };

    let (access, access_hash) = refresh_token::generate();
    let (refresh, refresh_hash) = refresh_token::generate();
    let now = Utc::now();
    mcp_tokens::insert_oauth_pair(
        user_id,
        &state.pool,
        &client_id,
        &IssuedTokenHashes {
            access: &access_hash,
            access_expires_at: now + ACCESS_TOKEN_TTL,
            refresh: &refresh_hash,
            refresh_expires_at: now + REFRESH_TOKEN_TTL,
        },
    )
    .await?;

    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({
            "access_token": access,
            "token_type": "Bearer",
            "expires_in": ACCESS_TOKEN_TTL.num_seconds(),
            "refresh_token": refresh,
        })),
    )
        .into_response())
}

async fn exchange_code(
    state: &AppState,
    request: TokenRequest,
) -> Result<(Uuid, String), OAuthError> {
    let (Some(code), Some(redirect_uri), Some(client_id), Some(verifier)) = (
        request.code,
        request.redirect_uri,
        request.client_id,
        request.code_verifier,
    ) else {
        return Err(OAuthError::bad_request(
            "invalid_request",
            "code, redirect_uri, client_id and code_verifier are required",
        ));
    };
    let invalid_grant = || OAuthError::bad_request("invalid_grant", "invalid or expired code");

    let consumed = oauth::consume_code(&state.pool, &refresh_token::hash_token(&code))
        .await?
        .ok_or_else(invalid_grant)?;
    let challenge_matches = (43..=128).contains(&verifier.len())
        && URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())) == consumed.code_challenge;
    if consumed.client_id != client_id
        || consumed.redirect_uri != redirect_uri
        || !challenge_matches
    {
        return Err(invalid_grant());
    }
    Ok((consumed.user_id, consumed.client_id))
}

async fn exchange_refresh_token(
    state: &AppState,
    request: TokenRequest,
) -> Result<(Uuid, String), OAuthError> {
    let Some(refresh) = request.refresh_token else {
        return Err(OAuthError::bad_request(
            "invalid_request",
            "refresh_token is required",
        ));
    };
    let invalid_grant =
        || OAuthError::bad_request("invalid_grant", "invalid or expired refresh token");
    let (user_id, client_id) =
        mcp_tokens::consume_refresh(&state.pool, &refresh_token::hash_token(&refresh))
            .await?
            .ok_or_else(invalid_grant)?;
    if request
        .client_id
        .is_some_and(|requested| requested != client_id)
    {
        return Err(invalid_grant());
    }
    Ok((user_id, client_id))
}

pub async fn list_clients(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<ConnectedClientResponse>>, ApiError> {
    Ok(Json(oauth::list_connected(user_id, &state.pool).await?))
}

pub async fn revoke_client(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(client_id): ApiPath<String>,
) -> Result<StatusCode, ApiError> {
    if oauth::revoke_client(user_id, &state.pool, &client_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// OAuth endpoints answer in RFC 6749's `{error, error_description}`
/// shape, not the app's usual `{error}` one - clients parse these.
pub struct OAuthError {
    status: StatusCode,
    error: &'static str,
    description: String,
}

impl OAuthError {
    fn bad_request(error: &'static str, description: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error,
            description: description.into(),
        }
    }
}

impl From<sqlx::Error> for OAuthError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(?error, "oauth error");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "server_error",
            description: "internal error".to_string(),
        }
    }
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        (
            self.status,
            [(header::CACHE_CONTROL, "no-store")],
            Json(json!({ "error": self.error, "error_description": self.description })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_uris_must_be_https_or_loopback() {
        assert!(is_allowed_redirect_uri(
            "https://claude.ai/api/mcp/auth_callback"
        ));
        assert!(is_allowed_redirect_uri("http://localhost:33418/callback"));
        assert!(is_allowed_redirect_uri("http://127.0.0.1/cb"));
        assert!(!is_allowed_redirect_uri("http://evil.example.com/cb"));
        assert!(!is_allowed_redirect_uri("javascript:alert(1)"));
        assert!(!is_allowed_redirect_uri("https://claude.ai/cb#fragment"));
        assert!(!is_allowed_redirect_uri("not a uri"));
    }

    #[test]
    fn resource_matches_mcp_endpoint_or_origin() {
        let public_url = "https://loom.example.com";
        assert!(is_this_resource(public_url, "https://loom.example.com/mcp"));
        assert!(is_this_resource(public_url, "https://loom.example.com/"));
        assert!(!is_this_resource(
            public_url,
            "https://other.example.com/mcp"
        ));
    }
}
