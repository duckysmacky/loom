use serde::Serialize;
use ts_rs::TS;

use super::active_period::ActivePeriodResponse;
use super::canvas::{CanvasArrowResponse, CanvasNoteResponse};
use super::edge::EdgeResponse;
use super::node::NodeResponse;
use super::poke::PokeResponse;

/// The `/api/board/canvas` response - the caller's entire graph (every
/// node and every edge, plus the canvas's sticky notes and annotation
/// arrows) in one payload. No filters: whole-graph view, any
/// client-side filtering is a Phase 11 frontend concern.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct CanvasResponse {
    pub nodes: Vec<NodeResponse>,
    pub edges: Vec<EdgeResponse>,
    pub notes: Vec<CanvasNoteResponse>,
    pub arrows: Vec<CanvasArrowResponse>,
}

/// The `/api/board/timeline` response - every poke and active period the
/// caller owns, across every node, in one payload (same whole-graph shape
/// as `CanvasResponse`).
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct TimelineResponse {
    pub pokes: Vec<PokeResponse>,
    pub periods: Vec<ActivePeriodResponse>,
}
