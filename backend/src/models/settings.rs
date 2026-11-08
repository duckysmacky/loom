use serde::{Deserialize, Serialize};
use ts_rs::TS;

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

/// Whether nodes at a given status (other than active) can go stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum StaleMode {
    /// Once untouched for the threshold (never-poked ones count from creation).
    Always,
    /// Only if it was poked at some point.
    Poked,
    Never,
}

/// Where auto-pause ends the open active period.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum StalePeriodEnd {
    /// At the last poke (or the period's start, if later): only real work counts.
    LastPoke,
    /// When the node went stale: the idle stretch stays in the period.
    StaleMoment,
}

/// How a path picks up staleness from the nodes inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum PathStaleMode {
    Off,
    /// Every node in play inside it is stale (done ones, and idea/queued/
    /// paused ones that aren't stale, are ignored).
    All,
    /// Its last touch (the latest poke inside it) is past the threshold.
    Any,
}

/// How a path's status is decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum PathStatusMode {
    /// Not shown; still derived underneath (blocking, staleness).
    Off,
    /// Set by hand, like any other node.
    Manual,
    /// Derived from the nodes inside it.
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum LineWidth {
    Thin,
    Normal,
    Thick,
}

/// How one kind of canvas connection is drawn.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(default)]
pub struct EdgeStyle {
    /// Hex color; `None` = the theme's default for this connection.
    pub color: Option<String>,
    pub line: LineStyle,
    pub width: LineWidth,
    pub arrow: bool,
}

impl EdgeStyle {
    fn new(line: LineStyle, width: LineWidth, arrow: bool) -> Self {
        Self {
            color: None,
            line,
            width,
            arrow,
        }
    }
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self::new(LineStyle::Solid, LineWidth::Normal, true)
    }
}

/// Canvas connection looks. `part_of` is drawn as the path box, so it has none.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(default)]
pub struct EdgeStyles {
    pub related: EdgeStyle,
    pub precedes: EdgeStyle,
    /// A `requires` edge whose requirement is not done yet.
    pub requires_unmet: EdgeStyle,
    /// A `requires` edge whose requirement is done.
    pub requires_met: EdgeStyle,
}

impl EdgeStyles {
    pub fn colors(&self) -> impl Iterator<Item = &str> {
        [
            &self.related,
            &self.precedes,
            &self.requires_unmet,
            &self.requires_met,
        ]
        .into_iter()
        .filter_map(|style| style.color.as_deref())
    }
}

impl Default for EdgeStyles {
    fn default() -> Self {
        Self {
            related: EdgeStyle::new(LineStyle::Dashed, LineWidth::Thin, false),
            precedes: EdgeStyle::new(LineStyle::Dashed, LineWidth::Thin, true),
            requires_unmet: EdgeStyle::new(LineStyle::Solid, LineWidth::Normal, true),
            requires_met: EdgeStyle::new(LineStyle::Solid, LineWidth::Normal, true),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "snake_case")]
pub enum FocusFill {
    Solid,
    /// Diagonal stripes of the colour and white.
    Striped,
    /// Transparent with a dashed border in the colour.
    Outline,
}

/// How a focus tier's bars are filled on the timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(default)]
pub struct FocusStyle {
    /// Hex color; `None` = the theme's default for this tier.
    pub color: Option<String>,
    pub fill: FocusFill,
}

impl Default for FocusStyle {
    fn default() -> Self {
        Self {
            color: None,
            fill: FocusFill::Solid,
        }
    }
}

/// Timeline bar looks: fill per focus tier, border per status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(default)]
pub struct TimelineStyles {
    pub primary: FocusStyle,
    pub secondary: FocusStyle,
    pub background: FocusStyle,
    /// Border of a done node's last bar; `None` = theme green.
    pub done_color: Option<String>,
    /// Dashed border of a blocked node's last bar; `None` = theme orange.
    pub blocked_color: Option<String>,
    /// Show paused nodes, with a dashed ghost bar over the time they have been paused.
    pub show_paused: bool,
}

impl TimelineStyles {
    pub fn colors(&self) -> impl Iterator<Item = &str> {
        [
            self.primary.color.as_deref(),
            self.secondary.color.as_deref(),
            self.background.color.as_deref(),
            self.done_color.as_deref(),
            self.blocked_color.as_deref(),
        ]
        .into_iter()
        .flatten()
    }
}

impl Default for TimelineStyles {
    fn default() -> Self {
        Self {
            primary: FocusStyle::default(),
            secondary: FocusStyle::default(),
            background: FocusStyle {
                color: None,
                fill: FocusFill::Striped,
            },
            done_color: None,
            blocked_color: None,
            show_paused: false,
        }
    }
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
    /// Snap canvas dragging to the grid.
    pub canvas_snap_to_grid: bool,
    /// Snap step in visible canvas cells: 2, 1, 1/2, 1/4 or 1/8.
    pub canvas_snap_step: f64,
    /// Dropping a node into a path box too small for it grows the box.
    pub path_auto_expand: bool,
    /// Canvas connections carry a small text label.
    pub show_edge_labels: bool,
    pub edge_styles: EdgeStyles,
    pub timeline_styles: TimelineStyles,
    /// Days without a poke (else since creation) before a node goes stale.
    #[ts(type = "number")]
    pub stale_days: i64,
    /// The dashboard warns this many days before a node goes stale; 0 = off.
    #[ts(type = "number")]
    pub stale_warn_days: i64,
    /// Which kinds can go stale (projects and/or studies; ideas never).
    pub stale_kinds: Vec<NodeKind>,
    /// Active nodes always can go stale; these cover the other statuses.
    pub stale_queued: StaleMode,
    pub stale_paused: StaleMode,
    pub stale_idea_status: StaleMode,
    /// An active node that goes stale is paused.
    pub stale_auto_pause: bool,
    pub stale_pause_period_end: StalePeriodEnd,
    pub path_stale_mode: PathStaleMode,
    pub path_status_mode: PathStatusMode,
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
            canvas_snap_to_grid: false,
            canvas_snap_step: 0.5,
            path_auto_expand: true,
            show_edge_labels: false,
            edge_styles: EdgeStyles::default(),
            timeline_styles: TimelineStyles::default(),
            stale_days: 14,
            stale_warn_days: 3,
            stale_kinds: vec![NodeKind::Project, NodeKind::Study],
            stale_queued: StaleMode::Never,
            stale_paused: StaleMode::Never,
            stale_idea_status: StaleMode::Never,
            stale_auto_pause: false,
            stale_pause_period_end: StalePeriodEnd::LastPoke,
            path_stale_mode: PathStaleMode::Any,
            path_status_mode: PathStatusMode::Auto,
        }
    }
}
