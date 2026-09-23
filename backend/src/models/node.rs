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

/// Derived child-completion ratio for a `part_of` container node - `done`/
/// `total` count of children pointing at it via a `part_of` edge. Distinct
/// from `progress_current`/`progress_total`, which are real user-editable
/// columns (e.g. a course's "15 of 30 videos").
#[derive(Debug, Clone, Copy, Serialize, TS)]
pub struct ContainerProgress {
    pub done: i64,
    pub total: i64,
}

/// A `nodes` row plus attached topic ids and derived graph state - the
/// JSON response shape for every node-returning endpoint. `user_id` is
/// deliberately not a field: ownership lives in the query's WHERE clause,
/// it never needs to round-trip to the client. Query targets don't map to
/// this directly once `container_progress` is involved (no single-column
/// SQL representation for a nested shape) - see `repo::nodes::NodeRow`.
#[derive(Debug, Clone, Serialize, TS)]
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
    pub blocked: bool,
    pub container_progress: Option<ContainerProgress>,
    pub last_poked_at: Option<DateTime<Utc>>,
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

/// A view preset for `GET /api/nodes` - translated into `kind`/`status`
/// constraints in Rust before the query runs (not a `sqlx::Type`: it's
/// never bound directly to a SQL parameter, unlike `NodeStatus`/`NodeFocus`/
/// `NodeKind`, which map 1:1 onto Postgres enum columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum NodeView {
    Backlog,
    All,
    Archived,
}

#[derive(Debug, Default, Deserialize, TS)]
pub struct NodeListQuery {
    #[serde(default)]
    pub status: Option<NodeStatus>,
    #[serde(default)]
    pub focus: Option<NodeFocus>,
    #[serde(default)]
    pub kind: Option<NodeKind>,
    #[serde(default)]
    pub view: Option<NodeView>,
}

#[derive(Debug, Deserialize, TS)]
pub struct AttachTopicRequest {
    pub topic_id: Uuid,
}
