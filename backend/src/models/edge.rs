use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[sqlx(type_name = "edge_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    Requires,
    PartOf,
    Related,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow, TS)]
pub struct EdgeResponse {
    pub id: Uuid,
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub kind: EdgeKind,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateEdgeRequest {
    pub from_node_id: Uuid,
    pub to_node_id: Uuid,
    pub kind: EdgeKind,
}

#[derive(Debug, Default, Deserialize, TS)]
pub struct EdgeListQuery {
    #[serde(default)]
    pub from_node_id: Option<Uuid>,
    #[serde(default)]
    pub to_node_id: Option<Uuid>,
    #[serde(default)]
    pub kind: Option<EdgeKind>,
}
