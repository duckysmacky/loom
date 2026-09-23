use axum::{Json, extract::State};

use super::error::ApiError;
use crate::middleware::auth_user::AuthUser;
use crate::models::board::CanvasResponse;
use crate::repo::board;
use crate::state::AppState;

pub async fn canvas(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<CanvasResponse>, ApiError> {
    let canvas = board::get_canvas(user_id, &state.pool).await?;
    Ok(Json(canvas))
}
