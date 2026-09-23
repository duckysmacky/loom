use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// One task on a node's checklist, ordered by `position` (append order).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct ChecklistItemResponse {
    pub id: Uuid,
    pub node_id: Uuid,
    pub title: String,
    pub done: bool,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateChecklistItemRequest {
    pub title: String,
}

#[derive(Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct UpdateChecklistItemRequest {
    #[serde(default)]
    #[ts(optional = nullable)]
    pub title: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub done: Option<bool>,
}
