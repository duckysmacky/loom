use serde_json::{Map, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::settings::Settings;

/// The account's settings; keys never saved fall back to the defaults.
pub async fn get(user_id: Uuid, pool: &PgPool) -> Result<Settings, sqlx::Error> {
    let stored = sqlx::query_scalar!("SELECT settings FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await?;
    Ok(serde_json::from_value(stored).unwrap_or_default())
}

/// Merges already-validated keys into the stored settings. Key-level (`||`),
/// so two devices changing different settings never overwrite each other.
pub async fn patch(
    user_id: Uuid,
    pool: &PgPool,
    changes: Map<String, Value>,
) -> Result<Settings, sqlx::Error> {
    let stored = sqlx::query_scalar!(
        "UPDATE users SET settings = settings || $2 WHERE id = $1 RETURNING settings",
        user_id,
        Value::Object(changes),
    )
    .fetch_one(pool)
    .await?;
    Ok(serde_json::from_value(stored).unwrap_or_default())
}
