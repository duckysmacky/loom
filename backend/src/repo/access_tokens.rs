use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::access_token::{AccessTokenResponse, TokenScope};

pub async fn create_personal(
    user_id: Uuid,
    pool: &PgPool,
    name: &str,
    token_hash: &str,
    scope: TokenScope,
    expires_at: Option<DateTime<Utc>>,
) -> Result<AccessTokenResponse, sqlx::Error> {
    sqlx::query_as!(
        AccessTokenResponse,
        r#"
        INSERT INTO access_tokens (user_id, kind, name, token_hash, scope, expires_at)
        VALUES ($1, 'personal', $2, $3, $4, $5)
        RETURNING id, name AS "name!", scope AS "scope: TokenScope",
                  created_at, last_used_at, expires_at
        "#,
        user_id,
        name,
        token_hash,
        scope as TokenScope,
        expires_at,
    )
    .fetch_one(pool)
    .await
}

pub async fn list_personal(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<AccessTokenResponse>, sqlx::Error> {
    sqlx::query_as!(
        AccessTokenResponse,
        r#"
        SELECT id, name AS "name!", scope AS "scope: TokenScope",
               created_at, last_used_at, expires_at
        FROM access_tokens
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
        "DELETE FROM access_tokens WHERE id = $1 AND user_id = $2 AND kind = 'personal'",
        token_id,
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Token kinds accepted on `/mcp`: personal tokens and OAuth access tokens.
pub const MCP_KINDS: &[&str] = &["personal", "access"];
/// Token kinds accepted on the REST API: only tokens the user made
/// themselves, never ones issued to OAuth connector clients.
pub const API_KINDS: &[&str] = &["personal"];

/// Who a presented token belongs to, and what it may do.
pub struct TokenIdentity {
    pub user_id: Uuid,
    pub scope: TokenScope,
}

/// Resolves a presented bearer token of one of `kinds` (never a refresh
/// token) to its owner, stamping `last_used_at` in the same statement.
/// The only function here not keyed by `user_id`: this is where the
/// `user_id` comes from.
pub async fn authenticate(
    pool: &PgPool,
    token_hash: &str,
    kinds: &[&str],
) -> Result<Option<TokenIdentity>, sqlx::Error> {
    sqlx::query_as!(
        TokenIdentity,
        r#"
        UPDATE access_tokens SET last_used_at = now()
        WHERE token_hash = $1
          AND kind = ANY($2)
          AND (expires_at IS NULL OR expires_at > now())
        RETURNING user_id, scope AS "scope: TokenScope"
        "#,
        token_hash,
        kinds as &[&str],
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
/// this they'd pile up forever). Personal tokens are left alone: an
/// expired one stays listed until the user revokes it.
pub async fn insert_oauth_pair(
    user_id: Uuid,
    pool: &PgPool,
    client_id: &str,
    hashes: &IssuedTokenHashes<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM access_tokens WHERE user_id = $1 AND kind <> 'personal' AND expires_at <= now()",
        user_id,
    )
    .execute(pool)
    .await?;
    sqlx::query!(
        r#"
        INSERT INTO access_tokens (user_id, kind, client_id, token_hash, expires_at)
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
        DELETE FROM access_tokens
        WHERE token_hash = $1 AND kind = 'refresh' AND expires_at > now()
        RETURNING user_id, client_id AS "client_id!"
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| (row.user_id, row.client_id)))
}
