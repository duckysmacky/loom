use serde::{Deserialize, Serialize};
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
    /// Archived nodes of any status; the other counts leave them out.
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
    pub study: i64,
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
    /// Nodes still in play (not done, not archived) with an unmet `requires` edge.
    #[ts(type = "number")]
    pub blocked: i64,
    /// Unpromoted ideas - same predicate as `GET /api/nodes?view=ideas`.
    #[ts(type = "number")]
    pub ideas: i64,
}

/// The `/api/dashboard` response - everything the dashboard shows, in one
/// payload ("control panel for what's up right now"). Only active nodes are
/// listed; everything else belongs to the board.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DashboardResponse {
    pub counts: DashboardCounts,
    pub stale: Vec<NodeResponse>,
    /// Every active node (paths by derived status), blocked included - by
    /// focus tier, unblocked before blocked within a tier.
    pub active: Vec<NodeResponse>,
    /// The newest unpromoted ideas, newest first.
    pub recent_ideas: Vec<NodeResponse>,
    /// `kind=path` nodes not done and not archived, newest first.
    pub paths: Vec<NodeResponse>,
}

/// Whether a queued/paused node can show up as stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum StaleMode {
    /// Once untouched for the threshold (never-poked ones count from creation).
    Always,
    /// Only if it was poked at some point.
    Poked,
    Never,
}

/// Query params for `GET /api/dashboard` tuning the stale list. Active nodes
/// are always eligible; omitted params fall back to the defaults.
#[derive(Debug, Default, Deserialize, TS)]
#[ts(export)]
pub struct DashboardQuery {
    #[serde(default)]
    #[ts(optional = nullable, type = "number")]
    pub stale_days: Option<i64>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub stale_queued: Option<StaleMode>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub stale_paused: Option<StaleMode>,
}
