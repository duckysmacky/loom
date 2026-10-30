use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::dashboard::{DashboardCounts, DashboardResponse, KindCounts, StatusCounts};
use crate::models::node::{NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus};
use crate::models::settings::Settings;
use crate::repo::{nodes, staleness};

const RECENT_IDEAS_LIMIT: usize = 5;

/// Assembles the whole dashboard, all carved out of one full node listing -
/// a single-user graph is small, and every extra `node_rows` read re-derives
/// every path rollup.
pub async fn get_dashboard(
    user_id: Uuid,
    pool: &PgPool,
    settings: &Settings,
) -> Result<DashboardResponse, sqlx::Error> {
    let all_nodes = nodes::list_nodes(user_id, pool, &NodeListQuery::default()).await?;
    let mut counts = count_nodes(&all_nodes);
    let (stale, going_stale) = stale_nodes(&all_nodes, settings);

    // Ideas (no status) aren't in play: they're off the board until promoted.
    // Archived nodes are hidden, whatever their status.
    let in_play = |node: &&NodeResponse| {
        !node.archived && !matches!(node.status, None | Some(NodeStatus::Done))
    };

    counts.blocked = all_nodes
        .iter()
        .filter(in_play)
        .filter(|node| node.blocked)
        .count() as i64;

    // Everything being worked on, paths aside (they have their own list):
    // primary tier first, then secondary, background; unblocked before
    // blocked within a tier.
    let mut active: Vec<NodeResponse> = all_nodes
        .iter()
        .filter(|node| {
            !node.archived && node.status == Some(NodeStatus::Active) && node.kind != NodeKind::Path
        })
        .cloned()
        .collect();
    // Stable sort - within a rank the listing's newest-first order holds.
    active.sort_by_key(|node| {
        let tier = match node.focus {
            Some(NodeFocus::Primary) => 0,
            Some(NodeFocus::Secondary) => 1,
            _ => 2,
        };
        (tier, node.blocked)
    });

    let recent_ideas = all_nodes
        .iter()
        .filter(|node| node.kind == NodeKind::Idea && !node.archived)
        .take(RECENT_IDEAS_LIMIT)
        .cloned()
        .collect();

    let paths = all_nodes
        .iter()
        .filter(in_play)
        .filter(|node| node.kind == NodeKind::Path)
        .cloned()
        .collect();

    Ok(DashboardResponse {
        counts,
        stale,
        going_stale,
        active,
        recent_ideas,
        paths,
    })
}

fn count_nodes(all_nodes: &[NodeResponse]) -> DashboardCounts {
    let mut by_status = StatusCounts {
        idea: 0,
        queued: 0,
        active: 0,
        paused: 0,
        done: 0,
        archived: 0,
    };
    let mut by_kind = KindCounts {
        idea: 0,
        project: 0,
        study: 0,
        path: 0,
    };

    for node in all_nodes {
        // Archived nodes are counted apart, not under the status they kept.
        if node.archived {
            by_status.archived += 1;
        } else {
            match node.status {
                Some(NodeStatus::Idea) => by_status.idea += 1,
                Some(NodeStatus::Queued) => by_status.queued += 1,
                Some(NodeStatus::Active) => by_status.active += 1,
                Some(NodeStatus::Paused) => by_status.paused += 1,
                Some(NodeStatus::Done) => by_status.done += 1,
                // Ideas have no status.
                None => {}
            }
        }
        match node.kind {
            NodeKind::Idea => by_kind.idea += 1,
            NodeKind::Project => by_kind.project += 1,
            NodeKind::Study => by_kind.study += 1,
            NodeKind::Path => by_kind.path += 1,
        }
    }

    DashboardCounts {
        total: all_nodes.len() as i64,
        by_status,
        by_kind,
        // Filled in by `get_dashboard` from the in-play nodes.
        blocked: 0,
        ideas: by_kind.idea,
    }
}

/// Recorded-stale nodes, oldest-touched first (most urgently stale first),
/// and the ones going stale within `stale_warn_days`, soonest first.
fn stale_nodes(
    all_nodes: &[NodeResponse],
    settings: &Settings,
) -> (Vec<NodeResponse>, Vec<NodeResponse>) {
    let last_touched = |node: &NodeResponse| node.last_poked_at.unwrap_or(node.created_at);
    let mut stale: Vec<NodeResponse> = all_nodes
        .iter()
        .filter(|node| node.stale_since.is_some() && !node.archived)
        .cloned()
        .collect();
    stale.sort_by_key(last_touched);

    let mut going_stale: Vec<NodeResponse> = Vec::new();
    if settings.stale_warn_days > 0 {
        let warn_from = Utc::now() + Duration::days(settings.stale_warn_days);
        going_stale = all_nodes
            .iter()
            .filter(|node| node.stale_since.is_none())
            .filter(|node| staleness::can_go_stale(node, settings))
            .filter(|node| staleness::stale_at(node, settings) < warn_from)
            .cloned()
            .collect();
        going_stale.sort_by_key(last_touched);
    }
    (stale, going_stale)
}
