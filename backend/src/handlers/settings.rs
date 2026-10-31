use axum::{Json, extract::State};
use serde_json::{Map, Value};

use super::error::ApiError;
use super::extract::ApiJson;
use super::validate_color;
use crate::middleware::auth_user::AuthUser;
use crate::models::node::NodeKind;
use crate::models::settings::{MAX_RECENT_COLORS, PathStatusMode, Settings};
use crate::repo::{nodes, settings, staleness};
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
    let parsed = validate_changes(&changes)?;
    // Switching to manual path statuses starts each path from the status it
    // had derived - read before the mode flips.
    if changes.contains_key("path_status_mode")
        && parsed.path_status_mode == PathStatusMode::Manual
        && settings::get(user_id, &state.pool).await?.path_status_mode != PathStatusMode::Manual
    {
        nodes::store_derived_path_statuses(user_id, &state.pool).await?;
    }
    let settings = settings::patch(user_id, &state.pool, changes).await?;
    // Thresholds, kinds and modes decide what's stale right now.
    staleness::sweep(user_id, &state.pool, &settings).await?;
    Ok(Json(settings))
}

/// Returns the changes typed (over the defaults, not the stored settings).
fn validate_changes(changes: &Map<String, Value>) -> Result<Settings, ApiError> {
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
    if !(0..=365).contains(&parsed.stale_warn_days) {
        return Err(ApiError::InvalidInput("stale_warn_days must be 0-365"));
    }
    if ![2.0, 1.0, 0.5, 0.25, 0.125].contains(&parsed.canvas_snap_step) {
        return Err(ApiError::InvalidInput("invalid canvas_snap_step"));
    }
    if parsed
        .stale_kinds
        .iter()
        .any(|kind| !matches!(kind, NodeKind::Project | NodeKind::Study))
    {
        return Err(ApiError::InvalidInput(
            "stale_kinds may only hold project and study",
        ));
    }
    if parsed.recent_colors.len() > MAX_RECENT_COLORS {
        return Err(ApiError::InvalidInput("too many recent colors"));
    }
    parsed
        .recent_colors
        .iter()
        .try_for_each(|color| validate_color(color))?;
    Ok(parsed)
}
