use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::dashboard::StaleMode;
use super::node::{NodeFocus, NodeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum Density {
    Comfortable,
    Compact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum IconStyle {
    Icons,
    Glyphs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum BoardView {
    Organized,
    Canvas,
    Timeline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum Grouping {
    Focus,
    Kind,
    Status,
    Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum PathDetailView {
    List,
    Canvas,
}

pub const MAX_RECENT_COLORS: usize = 8;

/// Per-account settings, stored as one jsonb column on `users`. Missing keys
/// fall back to `Default`, so adding a setting needs no migration. Behavior
/// settings (period tracking, stale thresholds) are read by the server itself
/// so REST, MCP and web callers all follow the account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(default)]
pub struct Settings {
    pub theme: Theme,
    pub density: Density,
    /// Lucide icons across the UI, or the older typographic glyphs.
    pub icon_style: IconStyle,
    pub default_board_view: BoardView,
    pub capture_kind: NodeKind,
    pub capture_focus: NodeFocus,
    pub poke_from_cards: bool,
    pub organized_grouping: Grouping,
    /// Custom colors picked via the native color input, most recent first.
    pub recent_colors: Vec<String>,
    /// A poke also switches the node's status to Active.
    pub poke_sets_active: bool,
    /// Pausing closes the active period, reactivating opens a new one.
    pub track_active_periods: bool,
    /// A focus change closes the open active period and starts a new one.
    pub split_period_on_focus_change: bool,
    pub path_detail_view: PathDetailView,
    /// The canvas lays out and pins unplaced nodes itself.
    pub auto_place: bool,
    /// Dropping a node into a path box too small for it grows the box.
    pub path_auto_expand: bool,
    /// Canvas connections carry a small text label.
    pub show_edge_labels: bool,
    /// Dashboard stale list: days without a poke before a node counts.
    #[ts(type = "number")]
    pub stale_days: i64,
    pub stale_queued: StaleMode,
    pub stale_paused: StaleMode,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            density: Density::Comfortable,
            icon_style: IconStyle::Icons,
            default_board_view: BoardView::Organized,
            capture_kind: NodeKind::Idea,
            capture_focus: NodeFocus::Secondary,
            poke_from_cards: true,
            organized_grouping: Grouping::Focus,
            recent_colors: Vec::new(),
            poke_sets_active: false,
            track_active_periods: true,
            split_period_on_focus_change: false,
            path_detail_view: PathDetailView::List,
            auto_place: false,
            path_auto_expand: true,
            show_edge_labels: false,
            stale_days: 14,
            stale_queued: StaleMode::Poked,
            stale_paused: StaleMode::Never,
        }
    }
}
