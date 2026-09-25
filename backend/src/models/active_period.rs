use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::deserialize_some;

/// One active/paused span on a node's history - additive to the node's own
/// `started_at`/`completed_at`, never a replacement. `ended_at` null means
/// still open (ongoing).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ActivePeriodResponse {
    pub id: Uuid,
    pub node_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateActivePeriodRequest {
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct UpdateActivePeriodRequest {
    /// Omit to leave unchanged; the column itself is never nullable.
    #[serde(default)]
    #[ts(optional = nullable)]
    pub started_at: Option<DateTime<Utc>>,
    /// Omit = unchanged, `null` = reopen (clear), a value = close/move it.
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub ended_at: Option<Option<DateTime<Utc>>>,
}
