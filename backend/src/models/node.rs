use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::deserialize_some;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[ts(export)]
#[sqlx(type_name = "node_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Idea,
    Project,
    Study,
    Path,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[ts(export)]
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
#[ts(export)]
#[sqlx(type_name = "node_focus", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NodeFocus {
    Primary,
    Secondary,
    Background,
}

/// A derived done/total ratio: `container_progress` counts `part_of` children,
/// `checklist_progress` counts checklist items. Distinct from
/// `progress_current`/`progress_total`, which are real user-editable columns
/// (e.g. a study's "15 of 30 videos").
#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export)]
pub struct Progress {
    #[ts(type = "number")]
    pub done: i64,
    #[ts(type = "number")]
    pub total: i64,
}

/// A `nodes` row plus attached topic ids and derived graph state - the
/// JSON response shape for every node-returning endpoint. `user_id` is
/// deliberately not a field: ownership lives in the query's WHERE clause,
/// it never needs to round-trip to the client. Query targets don't map to
/// this directly once `container_progress` is involved (no single-column
/// SQL representation for a nested shape) - see `repo::nodes::NodeRow`.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct NodeResponse {
    pub id: Uuid,
    pub kind: NodeKind,
    pub status: NodeStatus,
    pub focus: NodeFocus,
    pub title: String,
    pub progress_current: Option<i32>,
    pub progress_total: Option<i32>,
    /// What the progress counts ("videos", "chapters"); display only.
    pub progress_unit: Option<String>,
    pub color: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    /// Last position the node was dragged to on the board canvas; both
    /// `null` until it's placed by hand (the frontend auto-lays it out).
    pub canvas_x: Option<f64>,
    pub canvas_y: Option<f64>,
    pub topic_ids: Vec<Uuid>,
    pub blocked: bool,
    pub container_progress: Option<Progress>,
    pub checklist_progress: Option<Progress>,
    pub last_poked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateNodeRequest {
    pub kind: NodeKind,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub status: Option<NodeStatus>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub focus: Option<NodeFocus>,
    pub title: String,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub progress_current: Option<i32>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub progress_total: Option<i32>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub progress_unit: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub color: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub notes: Option<String>,
}

#[derive(Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct UpdateNodeRequest {
    #[serde(default)]
    #[ts(optional = nullable)]
    pub kind: Option<NodeKind>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub status: Option<NodeStatus>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub focus: Option<NodeFocus>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub progress_current: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub progress_total: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub progress_unit: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub color: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub notes: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub started_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub completed_at: Option<Option<DateTime<Utc>>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub canvas_x: Option<Option<f64>>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub canvas_y: Option<Option<f64>>,
}

/// A view preset for `GET /api/nodes` - translated into `kind`/`status`
/// constraints in Rust before the query runs (not a `sqlx::Type`: it's
/// never bound directly to a SQL parameter, unlike `NodeStatus`/`NodeFocus`/
/// `NodeKind`, which map 1:1 onto Postgres enum columns).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum NodeView {
    Backlog,
    All,
    Archived,
}

#[derive(Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct NodeListQuery {
    #[serde(default)]
    #[ts(optional = nullable)]
    pub status: Option<NodeStatus>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub focus: Option<NodeFocus>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub kind: Option<NodeKind>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub view: Option<NodeView>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct AttachTopicRequest {
    pub topic_id: Uuid,
}
