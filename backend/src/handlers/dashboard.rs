use axum::{Json, extract::State};

use super::error::ApiError;
use crate::middleware::auth_user::AuthUser;
use crate::models::dashboard::DashboardResponse;
use crate::repo::{dashboard, settings};
use crate::state::AppState;

pub async fn get(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<DashboardResponse>, ApiError> {
    let settings = settings::get(user_id, &state.pool).await?;
    Ok(Json(
        dashboard::get_dashboard(user_id, &state.pool, &settings).await?,
    ))
}
