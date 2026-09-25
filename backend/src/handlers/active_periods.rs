use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath};
use crate::middleware::auth_user::AuthUser;
use crate::models::active_period::{
    ActivePeriodResponse, CreateActivePeriodRequest, UpdateActivePeriodRequest,
};
use crate::repo::active_periods;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
) -> Result<Json<Vec<ActivePeriodResponse>>, ApiError> {
    let periods = active_periods::list_periods(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(periods))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
    ApiJson(request): ApiJson<CreateActivePeriodRequest>,
) -> Result<(StatusCode, Json<ActivePeriodResponse>), ApiError> {
    if let Some(ended_at) = request.ended_at
        && ended_at < request.started_at
    {
        return Err(ApiError::InvalidInput("ended_at must not be before started_at"));
    }
    let period = active_periods::create_period(
        user_id,
        &state.pool,
        node_id,
        request.started_at,
        request.ended_at,
    )
    .await?
    .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(period)))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(period_id): ApiPath<Uuid>,
    ApiJson(request): ApiJson<UpdateActivePeriodRequest>,
) -> Result<Json<ActivePeriodResponse>, ApiError> {
    let ended_at_set = request.ended_at.is_some();
    let ended_at = request.ended_at.flatten();
    let period = active_periods::update_period(
        user_id,
        &state.pool,
        period_id,
        request.started_at,
        ended_at_set,
        ended_at,
    )
    .await
    .map_err(map_period_error)?
    .ok_or(ApiError::NotFound)?;
    Ok(Json(period))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(period_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    if active_periods::delete_period(user_id, &state.pool, period_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Maps the `active_periods` CHECK constraint (`ended_at >= started_at`) to
/// a 400 - the DB already enforces it, this just surfaces it sensibly for
/// `update` (where `create`'s own synchronous check can't apply: only one
/// of the two dates may be present in the request).
fn map_period_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_check_violation()
    {
        return ApiError::InvalidInput("ended_at must not be before started_at");
    }
    ApiError::Internal(error.into())
}
