use chrono::{DateTime, Utc};
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

pub struct IssuedTokenHashes<'a> {
    pub access: &'a str,
    pub access_expires_at: DateTime<Utc>,
    pub refresh: &'a str,
    pub refresh_expires_at: DateTime<Utc>,
}

/// Stores a fresh OAuth access/refresh pair for `client_id`, clearing this
/// user's expired tokens on the way (access tokens are hourly - without
/// this they'd pile up forever).
pub async fn insert_oauth_pair(
    user_id: Uuid,
    pool: &PgPool,
    client_id: &str,
    hashes: &IssuedTokenHashes<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM mcp_tokens WHERE user_id = $1 AND expires_at <= now()",
        user_id,
    )
    .execute(pool)
    .await?;
    sqlx::query!(
        r#"
        INSERT INTO mcp_tokens (user_id, kind, client_id, token_hash, expires_at)
        VALUES ($1, 'access', $2, $3, $4), ($1, 'refresh', $2, $5, $6)
        "#,
        user_id,
        client_id,
        hashes.access,
        hashes.access_expires_at,
        hashes.refresh,
        hashes.refresh_expires_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Deletes and returns an unexpired refresh token's owner and client in
/// one statement - rotation: each refresh token works exactly once.
pub async fn consume_refresh(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<(Uuid, String)>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        DELETE FROM mcp_tokens
        WHERE token_hash = $1 AND kind = 'refresh' AND expires_at > now()
        RETURNING user_id, client_id AS "client_id!"
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| (row.user_id, row.client_id)))
}
