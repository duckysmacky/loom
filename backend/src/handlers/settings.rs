use axum::{Json, extract::State};
use serde_json::{Map, Value};

use super::error::ApiError;
use super::extract::ApiJson;
use super::validate_color;
use crate::middleware::auth_user::AuthUser;
use crate::models::settings::{MAX_RECENT_COLORS, Settings};
use crate::repo::settings;
use crate::state::AppState;

pub async fn get(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<Settings>, ApiError> {
    Ok(Json(settings::get(user_id, &state.pool).await?))
}

/// Partial update: only the keys sent change. Unknown keys and bad values are
/// rejected before anything is stored.
pub async fn patch(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    ApiJson(changes): ApiJson<Map<String, Value>>,
) -> Result<Json<Settings>, ApiError> {
    validate_changes(&changes)?;
    Ok(Json(settings::patch(user_id, &state.pool, changes).await?))
}

fn validate_changes(changes: &Map<String, Value>) -> Result<(), ApiError> {
    let Value::Object(known) = serde_json::to_value(Settings::default()).expect("serializable")
    else {
        unreachable!("settings serialize to an object");
    };
    if changes.keys().any(|key| !known.contains_key(key)) {
        return Err(ApiError::InvalidInput("unknown setting"));
    }
    // Typing the merged object catches wrong value types and bad enum values.
    let mut merged = known;
    merged.extend(changes.clone());
    let parsed: Settings = serde_json::from_value(Value::Object(merged))
        .map_err(|_| ApiError::InvalidInput("invalid setting value"))?;
    if !(1..=365).contains(&parsed.stale_days) {
        return Err(ApiError::InvalidInput("stale_days must be 1-365"));
    }
    if parsed.recent_colors.len() > MAX_RECENT_COLORS {
        return Err(ApiError::InvalidInput("too many recent colors"));
    }
    parsed
        .recent_colors
        .iter()
        .try_for_each(|color| validate_color(color))
}
