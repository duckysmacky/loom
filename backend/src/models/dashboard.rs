use serde::Serialize;
use ts_rs::TS;

use super::node::NodeResponse;

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export)]
pub struct StatusCounts {
    #[ts(type = "number")]
    pub idea: i64,
    #[ts(type = "number")]
    pub queued: i64,
    #[ts(type = "number")]
    pub active: i64,
    #[ts(type = "number")]
    pub paused: i64,
    #[ts(type = "number")]
    pub done: i64,
    #[ts(type = "number")]
    pub archived: i64,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export)]
pub struct KindCounts {
    #[ts(type = "number")]
    pub idea: i64,
    #[ts(type = "number")]
    pub project: i64,
    #[ts(type = "number")]
    pub course: i64,
    #[ts(type = "number")]
    pub path: i64,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export)]
pub struct DashboardCounts {
    #[ts(type = "number")]
    pub total: i64,
    pub by_status: StatusCounts,
    pub by_kind: KindCounts,
    /// Nodes still in play (not done/archived) with an unmet `requires` edge.
    #[ts(type = "number")]
    pub blocked: i64,
    /// Unpromoted ideas - same predicate as `GET /api/nodes?view=backlog`.
    #[ts(type = "number")]
    pub backlog: i64,
}

/// The `/api/dashboard` response - everything the dashboard shows, in one
/// payload ("control panel for what's up right now"). Only the primary focus
/// tier is here; the other tiers belong to the board's Organized view.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DashboardResponse {
    pub counts: DashboardCounts,
    pub stale: Vec<NodeResponse>,
    /// Every `focus=primary` node not done/archived, blocked ones included -
    /// actionable (active + unblocked) first, then blocked, then the rest.
    pub primary: Vec<NodeResponse>,
    /// The newest unpromoted ideas, newest first.
    pub recent_backlog: Vec<NodeResponse>,
    /// `kind=path` nodes not done/archived, newest first.
    pub paths: Vec<NodeResponse>,
}
