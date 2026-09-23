use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use super::error::ApiError;
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
    Json(request): Json<CreateTopicRequest>,
) -> Result<(StatusCode, Json<TopicResponse>), ApiError> {
    let topic = topics::create_topic(user_id, &state.pool, &request)
        .await
        .map_err(map_topic_error)?;
    Ok((StatusCode::CREATED, Json(topic)))
}

pub async fn get(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(topic_id): Path<Uuid>,
) -> Result<Json<TopicResponse>, ApiError> {
    let topic = topics::get_topic(user_id, &state.pool, topic_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(topic))
}

pub async fn update(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(topic_id): Path<Uuid>,
    Json(request): Json<UpdateTopicRequest>,
) -> Result<Json<TopicResponse>, ApiError> {
    let topic = topics::update_topic(user_id, &state.pool, topic_id, &request)
        .await
        .map_err(map_topic_error)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(topic))
}

pub async fn delete(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    Path(topic_id): Path<Uuid>,
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
