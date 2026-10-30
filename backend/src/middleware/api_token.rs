use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::auth::refresh_token::hash_token;
use crate::handlers::access_tokens::PERSONAL_TOKEN_PREFIX;
use crate::handlers::error::ApiError;
use crate::middleware::auth_user::AuthUser;
use crate::repo::access_tokens;
use crate::state::AppState;

/// The raw value of an `Authorization: Bearer ...` header.
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_once(' '))
        .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
        .map(|(_, token)| token.trim())
}

/// Lets a personal access token (`loom_...`) stand in for a web-session
/// JWT. Layered onto the data routes only - the account routes (login,
/// password, tokens, OAuth clients) never get it, so a leaked token can't
/// reach them. Anything that isn't a `loom_` token passes through to the
/// normal JWT check in the `AuthUser` extractor.
///
/// Tokens issued to OAuth connector clients are not accepted here: they
/// are for `/mcp` only. A `read` token may only make read requests.
pub async fn authenticate(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(token) =
        bearer_token(request.headers()).filter(|token| token.starts_with(PERSONAL_TOKEN_PREFIX))
    else {
        return next.run(request).await;
    };

    let identity = match access_tokens::authenticate(
        &state.pool,
        &hash_token(token),
        access_tokens::API_KINDS,
    )
    .await
    {
        Ok(Some(identity)) => identity,
        Ok(None) => return ApiError::InvalidToken.into_response(),
        Err(error) => return ApiError::from(error).into_response(),
    };

    let is_read = matches!(
        *request.method(),
        Method::GET | Method::HEAD | Method::OPTIONS
    );
    if !is_read && !identity.scope.allows_write() {
        return ApiError::Forbidden("this token is read-only").into_response();
    }

    request.extensions_mut().insert(AuthUser {
        user_id: identity.user_id,
    });
    next.run(request).await
}
