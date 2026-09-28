use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::ApiPath;
use crate::middleware::auth_user::AuthUser;
use crate::models::node::NodeKind;
use crate::models::poke::PokeResponse;
use crate::repo::{nodes, pokes};
use crate::state::AppState;

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
) -> Result<(StatusCode, Json<PokeResponse>), ApiError> {
    let node = nodes::get_node(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if node.kind == NodeKind::Path {
        return Err(ApiError::InvalidInput(
            "a path's pokes come from the nodes inside it",
        ));
    }
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
