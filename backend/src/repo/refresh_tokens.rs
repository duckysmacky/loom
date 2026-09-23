use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct RefreshTokenRow {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        token_hash,
        expires_at,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// A row is valid if it exists, hasn't been revoked, and hasn't expired.
pub async fn find_valid_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<RefreshTokenRow>, sqlx::Error> {
    sqlx::query_as!(
        RefreshTokenRow,
        r#"
        SELECT id, user_id
        FROM refresh_tokens
        WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
}

pub async fn revoke(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1
        "#,
        id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
