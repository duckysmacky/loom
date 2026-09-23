use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

/// A single `pokes` row - "I worked on this" signal. No `node_id` field:
/// always read under a known `/nodes/{id}/pokes` URL, redundant to echo
/// back.
#[derive(Debug, Clone, Serialize, TS)]
pub struct PokeResponse {
    pub id: Uuid,
    pub poked_at: DateTime<Utc>,
}
