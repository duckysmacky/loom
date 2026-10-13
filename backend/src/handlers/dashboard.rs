use axum::{Json, extract::State};

use super::extract::ApiQuery;

use super::error::ApiError;
use crate::middleware::auth_user::AuthUser;
use crate::models::dashboard::{DashboardQuery, DashboardResponse};
use crate::repo::dashboard;
use crate::state::AppState;

pub async fn get(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiQuery(query): ApiQuery<DashboardQuery>,
) -> Result<Json<DashboardResponse>, ApiError> {
    Ok(Json(
        dashboard::get_dashboard(user_id, &state.pool, &query).await?,
    ))
}
