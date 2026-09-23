use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use super::error::ApiError;
use super::extract::{ApiJson, ApiPath};
use super::validate_color;
use crate::middleware::auth_user::AuthUser;
use crate::models::topic::{CreateTopicRequest, TopicResponse, UpdateTopicRequest};
use crate::repo::topics;
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Vec<TopicResponse>>, ApiError> {
    let topics = topics::list_topics(user_id, &state.pool).await?;
    Ok(Json(topics))
}

pub async fn create(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(mut request): ApiJson<CreateTopicRequest>,
) -> Result<(StatusCode, Json<TopicResponse>), ApiError> {
    request.name = request.name.trim().to_string();
    if request.name.is_empty() {
        return Err(ApiError::InvalidInput("name must not be empty"));
    }
    if let Some(color) = &request.color {
        validate_color(color)?;
    }

    let topic = topics::create_topic(user_id, &state.pool, &request)
        .await
        .map_err(map_topic_error)?;
    Ok((StatusCode::CREATED, Json(topic)))
}

pub async fn get(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(topic_id): ApiPath<Uuid>,
) -> Result<Json<TopicResponse>, ApiError> {
    let topic = topics::get_topic(user_id, &state.pool, topic_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(topic))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(topic_id): ApiPath<Uuid>,
    ApiJson(mut request): ApiJson<UpdateTopicRequest>,
) -> Result<Json<TopicResponse>, ApiError> {
    if let Some(name) = &mut request.name {
        *name = name.trim().to_string();
        if name.is_empty() {
            return Err(ApiError::InvalidInput("name must not be empty"));
        }
    }
    if let Some(Some(color)) = &request.color {
        validate_color(color)?;
    }

    let topic = topics::update_topic(user_id, &state.pool, topic_id, &request)
        .await
        .map_err(map_topic_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(topic))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiPath(topic_id): ApiPath<Uuid>,
) -> Result<StatusCode, ApiError> {
    let deleted = topics::delete_topic(user_id, &state.pool, topic_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

fn map_topic_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_unique_violation()
    {
        return ApiError::Conflict("topic name already exists");
    }
    ApiError::Internal(error.into())
}
