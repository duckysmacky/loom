use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

use crate::models::checklist::ChecklistItemResponse;
use crate::models::node::Progress;

/// `None` means `node_id` isn't owned by `user_id`; `Some(vec)` (possibly
/// empty) means it is - same explicit ownership check as `pokes::list_pokes`.
pub async fn list_items(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
) -> Result<Option<Vec<ChecklistItemResponse>>, sqlx::Error> {
    let owned = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM nodes WHERE id = $2 AND user_id = $1) AS "owned!""#,
        user_id,
        node_id,
    )
    .fetch_one(pool)
    .await?;
    if !owned {
        return Ok(None);
    }

    let items = sqlx::query_as!(
        ChecklistItemResponse,
        r#"
        SELECT c.id, c.node_id, c.title, c.done, c.position, c.created_at
        FROM checklist_items c
        JOIN nodes n ON n.id = c.node_id
        WHERE n.user_id = $1 AND c.node_id = $2
        ORDER BY c.position, c.created_at
        "#,
        user_id,
        node_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(Some(items))
}

/// Appends an item at the end of the node's checklist. `None` means the node
/// isn't owned by `user_id` (the INSERT ... SELECT matches no row).
pub async fn create_item(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
    title: &str,
) -> Result<Option<ChecklistItemResponse>, sqlx::Error> {
    sqlx::query_as!(
        ChecklistItemResponse,
        r#"
        INSERT INTO checklist_items (node_id, title, position)
        SELECT n.id, $3,
            COALESCE((SELECT MAX(position) + 1 FROM checklist_items WHERE node_id = n.id), 0)
        FROM nodes n
        WHERE n.id = $2 AND n.user_id = $1
        RETURNING id, node_id, title, done, position, created_at
        "#,
        user_id,
        node_id,
        title,
    )
    .fetch_optional(pool)
    .await
}

/// Partial update; `None` fields are left unchanged. `None` result means the
/// item doesn't exist or its node isn't owned by `user_id`.
pub async fn update_item(
    user_id: Uuid,
    pool: &PgPool,
    item_id: Uuid,
    title: Option<&str>,
    done: Option<bool>,
) -> Result<Option<ChecklistItemResponse>, sqlx::Error> {
    sqlx::query_as!(
        ChecklistItemResponse,
        r#"
        UPDATE checklist_items c SET
            title = COALESCE($3, c.title),
            done = COALESCE($4, c.done)
        FROM nodes n
        WHERE c.id = $2 AND n.id = c.node_id AND n.user_id = $1
        RETURNING c.id, c.node_id, c.title, c.done, c.position, c.created_at
        "#,
        user_id,
        item_id,
        title,
        done,
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_item(user_id: Uuid, pool: &PgPool, item_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        DELETE FROM checklist_items c
        USING nodes n
        WHERE c.id = $2 AND n.id = c.node_id AND n.user_id = $1
        "#,
        user_id,
        item_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

/// Done/total over the node's checklist - `None` when it has no items.
pub async fn progress_for_node(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
) -> Result<Option<Progress>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) AS "total!",
            COUNT(*) FILTER (WHERE c.done) AS "done!"
        FROM checklist_items c
        JOIN nodes n ON n.id = c.node_id
        WHERE n.user_id = $1 AND c.node_id = $2
        "#,
        user_id,
        node_id,
    )
    .fetch_one(pool)
    .await?;
    Ok((row.total > 0).then_some(Progress {
        done: row.done,
        total: row.total,
    }))
}

/// Drops every checklist item on a node (used when it stops being a project).
pub async fn delete_all_for_node(
    user_id: Uuid,
    connection: &mut PgConnection,
    node_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM checklist_items c
        USING nodes n
        WHERE c.node_id = $2 AND n.id = c.node_id AND n.user_id = $1
        "#,
        user_id,
        node_id,
    )
    .execute(connection)
    .await?;
    Ok(())
}
