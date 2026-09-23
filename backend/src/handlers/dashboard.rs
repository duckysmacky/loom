use axum::{Json, extract::State};

use super::error::ApiError;
use crate::middleware::auth_user::AuthUser;
use crate::models::dashboard::DashboardResponse;
use crate::repo::dashboard;
use crate::state::AppState;

pub async fn get(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<DashboardResponse>, ApiError> {
    let counts = dashboard::get_counts(user_id, &state.pool).await?;
    let stale = dashboard::get_stale(user_id, &state.pool).await?;
    let unblocked_primary = dashboard::get_unblocked_primary(user_id, &state.pool).await?;

    Ok(Json(DashboardResponse {
        counts,
        stale,
        unblocked_primary,
    }))
}
