use sqlx::PgPool;
use uuid::Uuid;

use crate::models::topic::{CreateTopicRequest, TopicResponse, UpdateTopicRequest};

pub async fn create_topic(
    user_id: Uuid,
    pool: &PgPool,
    request: &CreateTopicRequest,
) -> Result<TopicResponse, sqlx::Error> {
    sqlx::query_as!(
        TopicResponse,
        r#"
        INSERT INTO topics (user_id, name, color)
        VALUES ($1, $2, $3)
        RETURNING id, name, color, created_at, updated_at
        "#,
        user_id,
        request.name,
        request.color,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_topics(user_id: Uuid, pool: &PgPool) -> Result<Vec<TopicResponse>, sqlx::Error> {
    sqlx::query_as!(
        TopicResponse,
        r#"
        SELECT id, name, color, created_at, updated_at
        FROM topics
        WHERE user_id = $1
        ORDER BY name
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_topic(
    user_id: Uuid,
    pool: &PgPool,
    topic_id: Uuid,
) -> Result<Option<TopicResponse>, sqlx::Error> {
    sqlx::query_as!(
        TopicResponse,
        r#"
        SELECT id, name, color, created_at, updated_at
        FROM topics
        WHERE user_id = $1 AND id = $2
        "#,
        user_id,
        topic_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_topic(
    user_id: Uuid,
    pool: &PgPool,
    topic_id: Uuid,
    request: &UpdateTopicRequest,
) -> Result<Option<TopicResponse>, sqlx::Error> {
    let color_set = request.color.is_some();
    let color = request.color.clone().flatten();

    sqlx::query_as!(
        TopicResponse,
        r#"
        UPDATE topics SET
            name = COALESCE($3, name),
            color = CASE WHEN $4 THEN $5 ELSE color END,
            updated_at = now()
        WHERE user_id = $1 AND id = $2
        RETURNING id, name, color, created_at, updated_at
        "#,
        user_id,
        topic_id,
        request.name,
        color_set,
        color,
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_topic(
    user_id: Uuid,
    pool: &PgPool,
    topic_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM topics WHERE user_id = $1 AND id = $2",
        user_id,
        topic_id,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
