use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Outcome of atomically consuming (single-use rotating) a refresh token.
pub enum ConsumeOutcome {
    /// Token was valid and unrevoked - now revoked, safe to issue a new pair for this user.
    Consumed(Uuid),
    /// Token was already rotated away by an earlier refresh - someone is replaying an
    /// old token, which is a theft signal. Caller should revoke every token for this user.
    Reused(Uuid),
    /// No usable token: never existed, expired, or revoked outright (logout, password
    /// change, theft response). A plain 401, not a theft signal - a device holding a
    /// cookie from before a password change must not sign out the session that made it.
    Invalid,
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

/// Consumes a refresh token for rotation (`/refresh`): revokes it and marks it
/// rotated, so a later replay is recognised as theft.
pub async fn consume(pool: &PgPool, token_hash: &str) -> Result<ConsumeOutcome, sqlx::Error> {
    revoke_single(pool, token_hash, true).await
}

/// Revokes a refresh token outright (`/logout`) - not a rotation, so presenting
/// it again later is just a stale cookie.
pub async fn revoke(pool: &PgPool, token_hash: &str) -> Result<ConsumeOutcome, sqlx::Error> {
    revoke_single(pool, token_hash, false).await
}

/// Atomically revokes a refresh token by hash, in one statement so two
/// concurrent uses of the same token can't both pass (one loses the race
/// on the `revoked_at IS NULL` predicate). Distinguishes a replayed rotated
/// token from any other unusable one so the caller can react to the former
/// as a theft signal.
async fn revoke_single(
    pool: &PgPool,
    token_hash: &str,
    rotated: bool,
) -> Result<ConsumeOutcome, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        UPDATE refresh_tokens SET revoked_at = now(), rotated = $2
        WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > now()
        RETURNING user_id
        "#,
        token_hash,
        rotated,
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return Ok(ConsumeOutcome::Consumed(row.user_id));
    }

    let replayed_rotation = sqlx::query_scalar!(
        "SELECT user_id FROM refresh_tokens WHERE token_hash = $1 AND rotated",
        token_hash,
    )
    .fetch_optional(pool)
    .await?;

    Ok(match replayed_rotation {
        Some(user_id) => ConsumeOutcome::Reused(user_id),
        None => ConsumeOutcome::Invalid,
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
