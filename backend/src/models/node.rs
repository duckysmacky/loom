use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::deserialize_some;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[sqlx(type_name = "node_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Idea,
    Project,
    Course,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[sqlx(type_name = "node_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Idea,
    Queued,
    Active,
    Paused,
    Done,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[sqlx(type_name = "node_focus", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NodeFocus {
    Primary,
    Secondary,
    Background,
}

/// A `nodes` row plus its attached topic ids - doubles as both the
/// `query_as!` mapping target and the JSON response shape. Unlike `User`
/// (which hides `password_hash`), no node column is sensitive or unused by
/// the frontend, so there's no separate row/response split. `user_id` is
/// deliberately not a field: ownership lives in the query's WHERE clause,
/// it never needs to round-trip to the client.
#[derive(Debug, Clone, Serialize, sqlx::FromRow, TS)]
pub struct NodeResponse {
    pub id: Uuid,
    pub kind: NodeKind,
    pub status: NodeStatus,
    pub focus: NodeFocus,
    pub title: String,
    pub progress_current: Option<i32>,
    pub progress_total: Option<i32>,
    pub color: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub topic_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateNodeRequest {
    pub kind: NodeKind,
    #[serde(default)]
    pub status: Option<NodeStatus>,
    #[serde(default)]
    pub focus: Option<NodeFocus>,
    pub title: String,
    #[serde(default)]
    pub progress_current: Option<i32>,
    #[serde(default)]
    pub progress_total: Option<i32>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Default, Deserialize, TS)]
pub struct UpdateNodeRequest {
    #[serde(default)]
    pub kind: Option<NodeKind>,
    #[serde(default)]
    pub status: Option<NodeStatus>,
    #[serde(default)]
    pub focus: Option<NodeFocus>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub progress_current: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub progress_total: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub color: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub notes: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub started_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub completed_at: Option<Option<DateTime<Utc>>>,
}

#[derive(Debug, Default, Deserialize, TS)]
pub struct NodeListQuery {
    #[serde(default)]
    pub status: Option<NodeStatus>,
    #[serde(default)]
    pub focus: Option<NodeFocus>,
    #[serde(default)]
    pub kind: Option<NodeKind>,
}

#[derive(Debug, Deserialize, TS)]
pub struct AttachTopicRequest {
    pub topic_id: Uuid,
}
