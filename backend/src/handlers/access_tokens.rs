use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath};
use crate::auth::refresh_token;
use crate::middleware::auth_user::AuthUser;
use crate::models::access_token::{
    AccessTokenResponse, CreateAccessTokenRequest, CreatedAccessTokenResponse,
};
use crate::repo::access_tokens;
use crate::state::AppState;

/// Personal tokens are recognisable at a glance (and by secret scanners).
pub const PERSONAL_TOKEN_PREFIX: &str = "loom_";

const MAX_NAME_LEN: usize = 100;
const MAX_EXPIRY_DAYS: u32 = 3650;

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<AccessTokenResponse>>, ApiError> {
    let tokens = access_tokens::list_personal(user_id, &state.pool).await?;
    Ok(Json(tokens))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(request): ApiJson<CreateAccessTokenRequest>,
) -> Result<(StatusCode, Json<CreatedAccessTokenResponse>), ApiError> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err(ApiError::InvalidInput("name must not be empty"));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(ApiError::InvalidInput(
            "name must be at most 100 characters",
        ));
    }

    if request
        .expires_in_days
        .is_some_and(|days| !(1..=MAX_EXPIRY_DAYS).contains(&days))
    {
        return Err(ApiError::InvalidInput(
            "expires_in_days must be between 1 and 3650",
        ));
    }
    let expires_at = request
        .expires_in_days
        .map(|days| Utc::now() + Duration::days(days.into()));

    let (raw, _) = refresh_token::generate();
    let raw_token = format!("{PERSONAL_TOKEN_PREFIX}{raw}");
    let token = access_tokens::create_personal(
        user_id,
        &state.pool,
        name,
        &refresh_token::hash_token(&raw_token),
        request.scope,
        expires_at,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedAccessTokenResponse { token, raw_token }),
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(token_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    if access_tokens::delete_personal(user_id, &state.pool, token_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
