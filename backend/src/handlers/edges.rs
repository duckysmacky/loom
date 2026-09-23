use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath, ApiQuery};
use crate::middleware::auth_user::AuthUser;
use crate::models::edge::{CreateEdgeRequest, EdgeListQuery, EdgeResponse};
use crate::repo::edges::{self, CreateEdgeOutcome};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiQuery(filters): ApiQuery<EdgeListQuery>,
) -> Result<Json<Vec<EdgeResponse>>, ApiError> {
    let edges = edges::list_edges(user_id, &state.pool, &filters).await?;
    Ok(Json(edges))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(request): ApiJson<CreateEdgeRequest>,
) -> Result<(StatusCode, Json<EdgeResponse>), ApiError> {
    match edges::create_edge(user_id, &state.pool, &request)
        .await
        .map_err(map_edge_error)?
    {
        CreateEdgeOutcome::Created(edge) => Ok((StatusCode::CREATED, Json(edge))),
        CreateEdgeOutcome::NotFound => Err(ApiError::NotFound),
        CreateEdgeOutcome::WouldCreateCycle => Err(ApiError::Conflict("would create a cycle")),
        CreateEdgeOutcome::AlreadyExists => Err(ApiError::Conflict("edge already exists")),
        CreateEdgeOutcome::NotAPath => Err(ApiError::InvalidInput("only paths can contain nodes")),
        CreateEdgeOutcome::AlreadyInPath => Err(ApiError::Conflict("already inside a path")),
    }
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(edge_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    let deleted = edges::delete_edge(user_id, &state.pool, edge_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

/// Maps the edges_no_self_loop CHECK constraint to a 400 instead of the
/// generic 500 path - same idea as nodes.rs's map_node_error.
fn map_edge_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_check_violation()
    {
        return ApiError::InvalidInput("edge cannot connect a node to itself");
    }
    ApiError::Internal(error.into())
}
