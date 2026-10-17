use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath};
use crate::middleware::auth_user::AuthUser;
use crate::models::canvas::{
    CanvasArrowResponse, CanvasNoteResponse, CreateCanvasArrowRequest, CreateCanvasNoteRequest,
    UpdateCanvasNoteRequest,
};
use crate::models::node::NodeKind;
use crate::repo::{canvas, nodes};
use crate::state::AppState;

const MIN_NOTE_WIDTH: f64 = 80.0;
const MIN_NOTE_HEIGHT: f64 = 50.0;

fn check_position(values: &[Option<f64>]) -> Result<(), ApiError> {
    if values.iter().flatten().all(|value| value.is_finite()) {
        Ok(())
    } else {
        Err(ApiError::InvalidInput(
            "canvas position must be a finite number",
        ))
    }
}

fn check_size(width: Option<f64>, height: Option<f64>) -> Result<(), ApiError> {
    if width.is_some_and(|width| !(width >= MIN_NOTE_WIDTH && width.is_finite()))
        || height.is_some_and(|height| !(height >= MIN_NOTE_HEIGHT && height.is_finite()))
    {
        return Err(ApiError::InvalidInput(
            "a note must be at least 80 wide and 50 high",
        ));
    }
    Ok(())
}

/// A note can only sit inside one of the caller's paths.
async fn check_path(state: &AppState, user_id: Uuid, path_id: Uuid) -> Result<(), ApiError> {
    let path = nodes::get_node(user_id, &state.pool, path_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if path.kind != NodeKind::Path {
        return Err(ApiError::InvalidInput("notes can only sit inside a path"));
    }
    Ok(())
}

pub async fn create_note(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(request): ApiJson<CreateCanvasNoteRequest>,
) -> Result<(StatusCode, Json<CanvasNoteResponse>), ApiError> {
    check_position(&[Some(request.canvas_x), Some(request.canvas_y)])?;
    check_size(request.canvas_width, request.canvas_height)?;
    if let Some(path_id) = request.path_id {
        check_path(&state, user_id, path_id).await?;
    }
    let note = canvas::create_note(user_id, &state.pool, &request).await?;
    Ok((StatusCode::CREATED, Json(note)))
}

pub async fn update_note(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(note_id): ApiPath<Uuid>,
    ApiJson(request): ApiJson<UpdateCanvasNoteRequest>,
) -> Result<Json<CanvasNoteResponse>, ApiError> {
    check_position(&[request.canvas_x, request.canvas_y])?;
    check_size(request.canvas_width, request.canvas_height)?;
    if let Some(Some(path_id)) = request.path_id {
        check_path(&state, user_id, path_id).await?;
    }
    let note = canvas::update_note(user_id, &state.pool, note_id, &request)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(note))
}

pub async fn delete_note(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(note_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    if canvas::delete_note(user_id, &state.pool, note_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

pub async fn create_arrow(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(request): ApiJson<CreateCanvasArrowRequest>,
) -> Result<(StatusCode, Json<CanvasArrowResponse>), ApiError> {
    let from = request.from_node_id.or(request.from_note_id);
    let to = request.to_node_id.or(request.to_note_id);
    if request.from_node_id.is_some() == request.from_note_id.is_some()
        || request.to_node_id.is_some() == request.to_note_id.is_some()
    {
        return Err(ApiError::InvalidInput(
            "an arrow needs exactly one from_node_id/from_note_id and one to_node_id/to_note_id",
        ));
    }
    if from == to {
        return Err(ApiError::InvalidInput(
            "an arrow can't point at its own start",
        ));
    }
    let arrow = canvas::create_arrow(user_id, &state.pool, &request)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(arrow)))
}

pub async fn delete_arrow(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(arrow_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    if canvas::delete_arrow(user_id, &state.pool, arrow_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
