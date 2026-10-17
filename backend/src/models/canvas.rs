use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::deserialize_some;

/// A sticky note on the canvas - free text, not a graph node. Inside a path
/// (`path_id`) its position is relative to the path box's top-left.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct CanvasNoteResponse {
    pub id: Uuid,
    pub body: String,
    pub canvas_x: f64,
    pub canvas_y: f64,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub path_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct CreateCanvasNoteRequest {
    /// The note's text (plain, may span lines). May be empty.
    #[serde(default)]
    pub body: String,
    /// Left edge in canvas pixels; relative to the path box when `path_id` is set.
    pub canvas_x: f64,
    /// Top edge; relative to the path box (keep >= 56) when `path_id` is set.
    pub canvas_y: f64,
    /// Defaults to 200.
    #[serde(default)]
    #[ts(optional = nullable)]
    pub canvas_width: Option<f64>,
    /// Defaults to 140.
    #[serde(default)]
    #[ts(optional = nullable)]
    pub canvas_height: Option<f64>,
    /// A path node to put the note inside (it moves with the path box).
    #[serde(default)]
    #[ts(optional = nullable)]
    pub path_id: Option<Uuid>,
}

#[derive(Debug, Default, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct UpdateCanvasNoteRequest {
    #[serde(default)]
    #[ts(optional = nullable)]
    pub body: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub canvas_x: Option<f64>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub canvas_y: Option<f64>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub canvas_width: Option<f64>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub canvas_height: Option<f64>,
    /// A path to move the note into, or `null` to take it out to the top
    /// level. Send the position with it - it's relative to the new parent.
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    #[schemars(with = "Option<Uuid>")]
    pub path_id: Option<Option<Uuid>>,
}

/// The side of a canvas item an arrow end attaches to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS, JsonSchema)]
#[ts(export)]
#[sqlx(type_name = "canvas_side", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CanvasSide {
    Top,
    Right,
    Bottom,
    Left,
}

/// An annotation arrow between canvas items. Each end is exactly one node or
/// one note. Purely visual - no meaning in the graph.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct CanvasArrowResponse {
    pub id: Uuid,
    pub from_node_id: Option<Uuid>,
    pub from_note_id: Option<Uuid>,
    pub to_node_id: Option<Uuid>,
    pub to_note_id: Option<Uuid>,
    pub from_side: CanvasSide,
    pub to_side: CanvasSide,
    pub created_at: DateTime<Utc>,
}

/// Set exactly one `from_*` and exactly one `to_*`.
#[derive(Debug, Default, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct CreateCanvasArrowRequest {
    #[serde(default)]
    #[ts(optional = nullable)]
    pub from_node_id: Option<Uuid>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub from_note_id: Option<Uuid>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub to_node_id: Option<Uuid>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub to_note_id: Option<Uuid>,
    /// Side of the start item the arrow leaves from. Defaults to `right`.
    #[serde(default)]
    #[ts(optional = nullable)]
    pub from_side: Option<CanvasSide>,
    /// Side of the end item the arrow points into. Defaults to `left`.
    #[serde(default)]
    #[ts(optional = nullable)]
    pub to_side: Option<CanvasSide>,
}
