use sqlx::PgPool;
use uuid::Uuid;

use crate::models::mcp_token::McpTokenResponse;

pub async fn create_personal(
    user_id: Uuid,
    pool: &PgPool,
    name: &str,
    token_hash: &str,
) -> Result<McpTokenResponse, sqlx::Error> {
    sqlx::query_as!(
        McpTokenResponse,
        r#"
        INSERT INTO mcp_tokens (user_id, kind, name, token_hash)
        VALUES ($1, 'personal', $2, $3)
        RETURNING id, name AS "name!", created_at, last_used_at
        "#,
        user_id,
        name,
        token_hash,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_personal(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<McpTokenResponse>, sqlx::Error> {
    sqlx::query_as!(
        McpTokenResponse,
        r#"
        SELECT id, name AS "name!", created_at, last_used_at
        FROM mcp_tokens
        WHERE user_id = $1 AND kind = 'personal'
        ORDER BY created_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_personal(
    user_id: Uuid,
    pool: &PgPool,
    token_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM mcp_tokens WHERE id = $1 AND user_id = $2 AND kind = 'personal'",
        token_id,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Resolves a presented bearer token (personal or OAuth access - never a
/// refresh token) to its owner, stamping `last_used_at` in the same
/// statement. The only function here not keyed by `user_id`: this is
/// where the `user_id` comes from.
pub async fn authenticate(pool: &PgPool, token_hash: &str) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        UPDATE mcp_tokens SET last_used_at = now()
        WHERE token_hash = $1
          AND kind IN ('personal', 'access')
          AND (expires_at IS NULL OR expires_at > now())
        RETURNING user_id
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
}
