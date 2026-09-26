use sqlx::PgPool;
use uuid::Uuid;

use crate::models::poke::PokeResponse;

/// `None` means `node_id` isn't owned by `user_id` - a single
/// ownership-checked insert, leaner than the two-step EXISTS-then-insert
/// pattern `node_topics::attach_topic` needs, since a poke has no
/// idempotency distinction to report back (unlike "already attached").
pub async fn create_poke(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
) -> Result<Option<PokeResponse>, sqlx::Error> {
    sqlx::query_as!(
        PokeResponse,
        r#"
        INSERT INTO pokes (node_id)
        SELECT $2 WHERE EXISTS (SELECT 1 FROM nodes WHERE id = $2 AND user_id = $1)
        RETURNING id, poked_at
        "#,
        user_id,
        node_id,
    )
    .fetch_optional(pool)
    .await
}

/// `None` means `node_id` isn't owned by `user_id`; `Some(vec)` (possibly
/// empty) means it is. A plain SELECT can't itself report "not owned" the
/// way `create_poke`'s conditional INSERT can, so this needs an explicit
/// ownership check first.
pub async fn list_pokes(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
) -> Result<Option<Vec<PokeResponse>>, sqlx::Error> {
    let owned = sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM nodes WHERE id = $1 AND user_id = $2) AS "owned!""#,
        node_id,
        user_id,
    )
    .fetch_one(pool)
    .await?;

    if !owned {
        return Ok(None);
    }

    let pokes = sqlx::query_as!(
        PokeResponse,
        "SELECT id, poked_at FROM pokes WHERE node_id = $1 ORDER BY poked_at DESC",
        node_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(Some(pokes))
}
