use chrono::{DateTime, Utc};
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use super::active_period::ActivePeriodResponse;
use super::edge::EdgeResponse;
use super::node::NodeResponse;

/// The `/api/board/canvas` response - the caller's entire graph (every
/// node and every edge) in one payload. No filters: whole-graph view, any
/// client-side filtering is a Phase 11 frontend concern.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct CanvasResponse {
    pub nodes: Vec<NodeResponse>,
    pub edges: Vec<EdgeResponse>,
}

/// A poke, with the `node_id` `PokeResponse` deliberately omits (that one's
/// always read under a known `/nodes/{id}/pokes` URL; this bulk endpoint
/// has no such context).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct TimelinePokeResponse {
    pub id: Uuid,
    pub node_id: Uuid,
    pub poked_at: DateTime<Utc>,
}

/// The `/api/board/timeline` response - every poke and active period the
/// caller owns, across every node, in one payload (same whole-graph shape
/// as `CanvasResponse`).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct TimelineResponse {
    pub pokes: Vec<TimelinePokeResponse>,
    pub periods: Vec<ActivePeriodResponse>,
}
