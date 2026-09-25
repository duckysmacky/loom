use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::oauth::ConnectedClientResponse;

pub struct OAuthClient {
    pub client_name: String,
    pub redirect_uris: Vec<String>,
}

pub struct ConsumedCode {
    pub user_id: Uuid,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
}

/// Clients register themselves before any user is involved, so these two
/// aren't user-scoped - a client row grants nothing on its own.
pub async fn create_client(
    pool: &PgPool,
    client_id: &str,
    client_name: &str,
    redirect_uris: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO oauth_clients (client_id, client_name, redirect_uris) VALUES ($1, $2, $3)",
        client_id,
        client_name,
        redirect_uris,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_client(
    pool: &PgPool,
    client_id: &str,
) -> Result<Option<OAuthClient>, sqlx::Error> {
    sqlx::query_as!(
        OAuthClient,
        "SELECT client_name, redirect_uris FROM oauth_clients WHERE client_id = $1",
        client_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_code(
    user_id: Uuid,
    pool: &PgPool,
    code_hash: &str,
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM oauth_codes WHERE expires_at <= now()")
        .execute(pool)
        .await?;
    sqlx::query!(
        r#"
        INSERT INTO oauth_codes (code_hash, client_id, user_id, redirect_uri, code_challenge, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
        code_hash,
        client_id,
        user_id,
        redirect_uri,
        code_challenge,
        expires_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Deletes and returns an unexpired code in one statement, so a code can
/// be exchanged at most once even under concurrent requests.
pub async fn consume_code(
    pool: &PgPool,
    code_hash: &str,
) -> Result<Option<ConsumedCode>, sqlx::Error> {
    sqlx::query_as!(
        ConsumedCode,
        r#"
        DELETE FROM oauth_codes WHERE code_hash = $1 AND expires_at > now()
        RETURNING user_id, client_id, redirect_uri, code_challenge
        "#,
        code_hash,
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_connected(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<ConnectedClientResponse>, sqlx::Error> {
    sqlx::query_as!(
        ConnectedClientResponse,
        r#"
        SELECT c.client_id, c.client_name,
               MIN(t.created_at) AS "connected_at!", MAX(t.last_used_at) AS last_used_at
        FROM mcp_tokens t
        JOIN oauth_clients c ON c.client_id = t.client_id
        WHERE t.user_id = $1
        GROUP BY c.client_id, c.client_name
        ORDER BY MIN(t.created_at) DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await
}

/// Signs a connector out: every access and refresh token this user
/// granted to it. The client registration itself stays (it's shared,
/// and harmless without tokens).
pub async fn revoke_client(
    user_id: Uuid,
    pool: &PgPool,
    client_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM mcp_tokens WHERE user_id = $1 AND client_id = $2",
        user_id,
        client_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
