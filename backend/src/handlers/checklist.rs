use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath};
use crate::middleware::auth_user::AuthUser;
use crate::models::checklist::{
    ChecklistItemResponse, CreateChecklistItemRequest, UpdateChecklistItemRequest,
};
use crate::models::node::NodeKind;
use crate::repo::{checklist, nodes};
use crate::state::AppState;

fn clean_title(title: &str) -> Result<&str, ApiError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(ApiError::InvalidInput("title must not be empty"));
    }
    Ok(title)
}

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
) -> Result<Json<Vec<ChecklistItemResponse>>, ApiError> {
    let items = checklist::list_items(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(items))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(node_id): ApiPath<Uuid>,
    ApiJson(request): ApiJson<CreateChecklistItemRequest>,
) -> Result<(StatusCode, Json<ChecklistItemResponse>), ApiError> {
    let title = clean_title(&request.title)?;
    let node = nodes::get_node(user_id, &state.pool, node_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if node.kind != NodeKind::Project {
        return Err(ApiError::InvalidInput("only projects have a checklist"));
    }
    let item = checklist::create_item(user_id, &state.pool, node_id, title)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(item_id): ApiPath<Uuid>,
    ApiJson(request): ApiJson<UpdateChecklistItemRequest>,
) -> Result<Json<ChecklistItemResponse>, ApiError> {
    let title = request.title.as_deref().map(clean_title).transpose()?;
    let item = checklist::update_item(user_id, &state.pool, item_id, title, request.done)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(item))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(item_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    if checklist::delete_item(user_id, &state.pool, item_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}
