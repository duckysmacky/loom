use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// A single `pokes` row - "I worked on this" signal. Carries `node_id`
/// because a path's history and the timeline mix pokes of many nodes.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PokeResponse {
    pub id: Uuid,
    pub node_id: Uuid,
    pub poked_at: DateTime<Utc>,
}

/// Body of `POST /nodes/{id}/pokes`; the whole body is optional. `poked_at`
/// backdates the poke (default now) for work that was never logged.
#[derive(Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct CreatePokeRequest {
    #[serde(default)]
    #[ts(optional = nullable)]
    pub poked_at: Option<DateTime<Utc>>,
}
