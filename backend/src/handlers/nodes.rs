use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath, ApiQuery};
use super::validate_color;
use crate::middleware::auth_user::AuthUser;
use crate::models::node::{
    AttachTopicRequest, CreateNodeRequest, NodeListQuery, NodeResponse, UpdateNodeRequest,
};
use crate::repo::node_topics::{AttachOutcome, DetachOutcome};
use crate::repo::{node_topics, nodes};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiQuery(filters): ApiQuery<NodeListQuery>,
) -> Result<Json<Vec<NodeResponse>>, ApiError> {
    let nodes = nodes::list_nodes(user_id, &state.pool, &filters).await?;
    Ok(Json(nodes))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(mut request): ApiJson<CreateNodeRequest>,
) -> Result<(StatusCode, Json<NodeResponse>), ApiError> {
    request.title = request.title.trim().to_string();
    if request.title.is_empty() {
        return Err(ApiError::InvalidInput("title must not be empty"));
    }
    if let Some(color) = &request.color {
        validate_color(color)?;
    }

    let node = nodes::create_node(user_id, &state.pool, &request)
        .await
        .map_err(map_node_error)?;
    Ok((StatusCode::CREATED, Json(node)))
}

pub async fn get(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
) -> Result<Json<NodeResponse>, ApiError> {
    let node = nodes::get_node(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(node))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
    ApiJson(mut request): ApiJson<UpdateNodeRequest>,
) -> Result<Json<NodeResponse>, ApiError> {
    if let Some(title) = &mut request.title {
        *title = title.trim().to_string();
        if title.is_empty() {
            return Err(ApiError::InvalidInput("title must not be empty"));
        }
    }
    if let Some(Some(color)) = &request.color {
        validate_color(color)?;
    }

    let node = nodes::update_node(user_id, &state.pool, node_id, &request)
        .await
        .map_err(map_node_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(node))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    let deleted = nodes::delete_node(user_id, &state.pool, node_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn attach_topic(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
    ApiJson(request): ApiJson<AttachTopicRequest>,
) -> Result<StatusCode, ApiError> {
    match node_topics::attach_topic(user_id, &state.pool, node_id, request.topic_id).await? {
        AttachOutcome::Attached | AttachOutcome::AlreadyAttached => Ok(StatusCode::NO_CONTENT),
        AttachOutcome::NotFound => Err(ApiError::NotFound),
    }
}

pub async fn detach_topic(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath((node_id, topic_id)): ApiPath<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    match node_topics::detach_topic(user_id, &state.pool, node_id, topic_id).await? {
        DetachOutcome::Detached => Ok(StatusCode::NO_CONTENT),
        DetachOutcome::NotFound => Err(ApiError::NotFound),
    }
}

/// Maps the nodes.progress_current/progress_total CHECK constraint to a
/// 400 instead of falling into the generic 500 path - the DB already
/// enforces the invariant, this just surfaces it sensibly.
fn map_node_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_check_violation()
    {
        return ApiError::InvalidInput("progress_current/progress_total is invalid");
    }
    ApiError::Internal(error.into())
}
