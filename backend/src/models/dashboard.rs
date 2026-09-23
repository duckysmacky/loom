use serde::Serialize;
use ts_rs::TS;

use super::node::NodeResponse;

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export)]
pub struct StatusCounts {
    pub idea: i64,
    pub queued: i64,
    pub active: i64,
    pub paused: i64,
    pub done: i64,
    pub archived: i64,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export)]
pub struct KindCounts {
    pub idea: i64,
    pub project: i64,
    pub course: i64,
}

#[derive(Debug, Clone, Copy, Serialize, TS)]
#[ts(export)]
pub struct DashboardCounts {
    pub total: i64,
    pub by_status: StatusCounts,
    pub by_kind: KindCounts,
}

/// The `/api/dashboard` response - aggregated counts, the stale list, and
/// the unblocked-primary list, in one payload ("control panel for what's
/// up right now").
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct DashboardResponse {
    pub counts: DashboardCounts,
    pub stale: Vec<NodeResponse>,
    pub unblocked_primary: Vec<NodeResponse>,
}
