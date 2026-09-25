use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath};
use crate::auth::refresh_token;
use crate::middleware::auth_user::AuthUser;
use crate::models::mcp_token::{
    CreateMcpTokenRequest, CreatedMcpTokenResponse, McpInfoResponse, McpTokenResponse,
};
use crate::repo::mcp_tokens;
use crate::state::AppState;

/// Personal tokens are recognisable at a glance (and by secret scanners).
pub const PERSONAL_TOKEN_PREFIX: &str = "loom_";

const MAX_NAME_LEN: usize = 100;

pub async fn info(State(state): State<AppState>) -> Json<McpInfoResponse> {
    Json(McpInfoResponse {
        enabled: state.mcp_public_url.is_some(),
        url: state
            .mcp_public_url
            .as_ref()
            .map(|url| format!("{url}/mcp")),
    })
}

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<McpTokenResponse>>, ApiError> {
    let tokens = mcp_tokens::list_personal(user_id, &state.pool).await?;
    Ok(Json(tokens))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(request): ApiJson<CreateMcpTokenRequest>,
) -> Result<(StatusCode, Json<CreatedMcpTokenResponse>), ApiError> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err(ApiError::InvalidInput("name must not be empty"));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(ApiError::InvalidInput(
            "name must be at most 100 characters",
        ));
    }

    let (raw, _) = refresh_token::generate();
    let raw_token = format!("{PERSONAL_TOKEN_PREFIX}{raw}");
    let token = mcp_tokens::create_personal(
        user_id,
        &state.pool,
        name,
        &refresh_token::hash_token(&raw_token),
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedMcpTokenResponse { token, raw_token }),
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(token_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    if mcp_tokens::delete_personal(user_id, &state.pool, token_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
