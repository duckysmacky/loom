use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath, ApiQuery};
use super::validate_color;
use crate::middleware::auth_user::AuthUser;
use crate::models::node::{
    AttachTopicRequest, CreateNodeRequest, NodeKind, NodeListQuery, NodeResponse, NodeStatus,
    ReorderNodesRequest, UpdateNodeRequest,
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
    request.progress_unit = clean_progress_unit(request.progress_unit.take())?;
    let sets_progress = request.progress_current.is_some()
        || request.progress_total.is_some()
        || request.progress_unit.is_some();
    if sets_progress && request.kind != NodeKind::Study {
        return Err(ONLY_STUDY_PROGRESS);
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
    if let Some(unit) = request.progress_unit.take() {
        request.progress_unit = Some(clean_progress_unit(unit)?);
    }
    let sets_progress = matches!(request.progress_current, Some(Some(_)))
        || matches!(request.progress_total, Some(Some(_)))
        || matches!(request.progress_unit, Some(Some(_)));
    if sets_progress {
        let resulting_kind = match request.kind {
            Some(kind) => kind,
            None => {
                nodes::get_node(user_id, &state.pool, node_id)
                    .await?
                    .ok_or(ApiError::NotFound)?
                    .kind
            }
        };
        if resulting_kind != NodeKind::Study {
            return Err(ONLY_STUDY_PROGRESS);
        }
    }

    // A completed date is the last active period's end, so it only exists
    // on a done node that has started; clearing it elsewhere is a no-op.
    if let Some(completed_at) = request.completed_at {
        let current = nodes::get_node(user_id, &state.pool, node_id)
            .await?
            .ok_or(ApiError::NotFound)?;
        let done = request.status.unwrap_or(current.status) == NodeStatus::Done;
        let started = match request.started_at {
            Some(started_at) => started_at.is_some(),
            None => current.started_at.is_some(),
        };
        match completed_at {
            Some(_) if !done => {
                return Err(ApiError::InvalidInput(
                    "only done nodes have a completion date",
                ));
            }
            Some(_) if !started => {
                return Err(ApiError::InvalidInput(
                    "a node needs a start date before a completion date",
                ));
            }
            None if !done => request.completed_at = None,
            _ => {}
        }
    }

    let node = nodes::update_node(user_id, &state.pool, node_id, &request)
        .await
        .map_err(map_node_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(node))
}

pub async fn reorder(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(request): ApiJson<ReorderNodesRequest>,
) -> Result<StatusCode, ApiError> {
    if !nodes::reorder_nodes(user_id, &state.pool, &request).await? {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clear_order(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<StatusCode, ApiError> {
    nodes::clear_order(user_id, &state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
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

const MAX_PROGRESS_UNIT_LEN: usize = 40;

/// Kinds are strict: only study nodes carry a tracked progress counter.
const ONLY_STUDY_PROGRESS: ApiError = ApiError::InvalidInput("only study nodes track progress");

/// Trims the progress label; blank means "no label" (stored as NULL).
fn clean_progress_unit(unit: Option<String>) -> Result<Option<String>, ApiError> {
    let Some(unit) = unit.map(|unit| unit.trim().to_string()) else {
        return Ok(None);
    };
    if unit.chars().count() > MAX_PROGRESS_UNIT_LEN {
        return Err(ApiError::InvalidInput(
            "progress_unit must be at most 40 characters",
        ));
    }
    Ok((!unit.is_empty()).then_some(unit))
}

/// Maps the nodes CHECK constraints (progress pair, canvas position pair)
/// to a 400 instead of falling into the generic 500 path - the DB already
/// enforces the invariants, this just surfaces them sensibly.
fn map_node_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_check_violation()
    {
        match db_error.constraint() {
            Some("nodes_canvas_position_pair") => {
                return ApiError::InvalidInput("canvas_x/canvas_y must be set or cleared together");
            }
            Some("nodes_canvas_size_pair") => {
                return ApiError::InvalidInput(
                    "canvas_width/canvas_height must be set or cleared together",
                );
            }
            Some("nodes_canvas_size_positive") => {
                return ApiError::InvalidInput("canvas_width/canvas_height must be positive");
            }
            // Started/completed are the edges of the active periods, so a
            // start after its period's end trips the period's own check.
            Some("active_periods_check") => {
                return ApiError::InvalidInput("an active period can't end before it starts");
            }
            _ => {}
        }
        return ApiError::InvalidInput("progress_current/progress_total is invalid");
    }
    ApiError::Internal(error.into())
}
