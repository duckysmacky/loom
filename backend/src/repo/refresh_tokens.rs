use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Outcome of atomically consuming (single-use rotating) a refresh token.
pub enum ConsumeOutcome {
    /// Token was valid and unrevoked - now revoked, safe to issue a new pair for this user.
    Consumed(Uuid),
    /// Token hash matches a row that was already revoked or has expired - reuse of a
    /// rotated-out token, which is a theft signal. Caller should revoke every token for
    /// this user.
    Reused(Uuid),
    /// No row with this hash exists at all.
    NotFound,
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

/// Atomically revokes a refresh token by hash, in one statement so two
/// concurrent uses of the same token can't both pass (one loses the race
/// on the `revoked_at IS NULL` predicate). Distinguishes "already used /
/// expired" from "never existed" so the caller can react to reuse as a
/// theft signal.
pub async fn consume(pool: &PgPool, token_hash: &str) -> Result<ConsumeOutcome, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        UPDATE refresh_tokens SET revoked_at = now()
        WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()
        RETURNING user_id
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return Ok(ConsumeOutcome::Consumed(row.user_id));
    }

    let existing = sqlx::query_scalar!(
        "SELECT user_id FROM refresh_tokens WHERE token_hash = $1",
        token_hash,
    )
    .fetch_optional(pool)
    .await?;

    Ok(match existing {
        Some(user_id) => ConsumeOutcome::Reused(user_id),
        None => ConsumeOutcome::NotFound,
    })
}

/// Reuse of a rotated-out refresh token is treated as theft - revoke every
/// token this user has, forcing a fresh login everywhere.
pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL",
        user_id,
    )
    .execute(pool)
    .await?;

    Ok(())
}
