use sqlx::PgPool;
use uuid::Uuid;

pub enum AttachOutcome {
    Attached,
    AlreadyAttached,
    NotFound,
}

pub enum DetachOutcome {
    Detached,
    NotFound,
}

pub async fn attach_topic(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
    topic_id: Uuid,
) -> Result<AttachOutcome, sqlx::Error> {
    let owned = sqlx::query_scalar!(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM nodes WHERE id = $1 AND user_id = $3)
            AND EXISTS(SELECT 1 FROM topics WHERE id = $2 AND user_id = $3)
            AS "owned!"
        "#,
        node_id,
        topic_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    if !owned {
        return Ok(AttachOutcome::NotFound);
    }

    let result = sqlx::query!(
        "INSERT INTO node_topics (node_id, topic_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        node_id,
        topic_id,
    )
    .execute(pool)
    .await?;

    Ok(if result.rows_affected() == 1 {
        AttachOutcome::Attached
    } else {
        AttachOutcome::AlreadyAttached
    })
}

pub async fn detach_topic(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
    topic_id: Uuid,
) -> Result<DetachOutcome, sqlx::Error> {
    let node_owned = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1 AND user_id = $2) AS "owned!""#,
        node_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    if !node_owned {
        return Ok(DetachOutcome::NotFound);
    }

    // Idempotent: deleting a link that doesn't exist is still a success -
    // topic ownership doesn't need a separate check here, worst case this
    // deletes zero rows.
    sqlx::query!(
        "DELETE FROM node_topics WHERE node_id = $1 AND topic_id = $2",
        node_id,
        topic_id,
    )
    .execute(pool)
    .await?;

    Ok(DetachOutcome::Detached)
}
