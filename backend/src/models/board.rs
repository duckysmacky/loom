use serde::Serialize;
use ts_rs::TS;

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
