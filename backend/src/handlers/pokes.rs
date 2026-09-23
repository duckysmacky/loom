use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::ApiPath;
use crate::middleware::auth_user::AuthUser;
use crate::models::poke::PokeResponse;
use crate::repo::pokes;
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
) -> Result<(StatusCode, Json<PokeResponse>), ApiError> {
    let poke = pokes::create_poke(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(poke)))
}

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
) -> Result<Json<Vec<PokeResponse>>, ApiError> {
    let pokes = pokes::list_pokes(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(pokes))
}
