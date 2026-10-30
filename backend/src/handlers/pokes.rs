use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use chrono::{Duration, Utc};

use super::error::ApiError;
use super::extract::ApiPath;
use crate::middleware::auth_user::AuthUser;
use crate::models::node::{NodeKind, NodeStatus, UpdateNodeRequest};
use crate::models::poke::{CreatePokeRequest, PokeResponse};
use crate::repo::{nodes, pokes, settings, staleness};
use crate::state::AppState;

/// How far ahead of the server clock a `poked_at` may be - covers browser
/// clock skew when the user picks "now".
const FUTURE_GRACE: Duration = Duration::seconds(60);

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
    request: Option<Json<CreatePokeRequest>>,
) -> Result<(StatusCode, Json<PokeResponse>), ApiError> {
    let poked_at = request.and_then(|Json(request)| request.poked_at);
    if poked_at.is_some_and(|poked_at| poked_at > Utc::now() + FUTURE_GRACE) {
        return Err(ApiError::InvalidInput("poked_at must not be in the future"));
    }
    let node = nodes::get_node(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if node.kind == NodeKind::Path {
        return Err(ApiError::InvalidInput(
            "a path's pokes come from the nodes inside it",
        ));
    }
    let poke = pokes::create_poke(user_id, &state.pool, node_id, poked_at)
        .await?
        .ok_or(ApiError::NotFound)?;

    // A poke (live or backdated) that brings the last touch back within the
    // threshold un-stales the node; with "poke sets active", a paused one
    // resumes too.
    let settings = settings::get(user_id, &state.pool).await?;
    staleness::sweep(user_id, &state.pool, &settings).await?;
    if node.stale_since.is_some()
        && node.status == Some(NodeStatus::Paused)
        && settings.poke_sets_active
    {
        let unstaled = nodes::get_node(user_id, &state.pool, node_id)
            .await?
            .is_some_and(|node| node.stale_since.is_none());
        if unstaled {
            let resume = UpdateNodeRequest {
                status: Some(NodeStatus::Active),
                ..Default::default()
            };
            nodes::update_node(user_id, &state.pool, node_id, &resume, &settings).await?;
        }
    }
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
