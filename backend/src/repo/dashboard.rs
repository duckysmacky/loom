use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::dashboard::{
    DashboardCounts, DashboardQuery, DashboardResponse, KindCounts, StaleMode, StatusCounts,
};
use crate::models::node::{NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus};
use crate::repo::nodes;

const RECENT_IDEAS_LIMIT: usize = 5;
const STALE_AFTER_DAYS: i64 = 14;

/// Assembles the whole dashboard, all carved out of one full node listing -
/// a single-user graph is small, and every extra `node_rows` read re-derives
/// every path rollup.
pub async fn get_dashboard(
    user_id: Uuid,
    pool: &PgPool,
    query: &DashboardQuery,
) -> Result<DashboardResponse, sqlx::Error> {
    let all_nodes = nodes::list_nodes(user_id, pool, &NodeListQuery::default()).await?;
    let mut counts = count_nodes(&all_nodes);
    let stale = stale_nodes(&all_nodes, query);

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

/// Nodes untouched (no poke, else created) for the threshold days. Active
/// ones always qualify; queued/paused per `StaleMode`; done/archived/ideas
/// never. Oldest-touched first (most urgently stale first).
fn stale_nodes(all_nodes: &[NodeResponse], query: &DashboardQuery) -> Vec<NodeResponse> {
    let days = query.stale_days.unwrap_or(STALE_AFTER_DAYS).clamp(1, 365);
    let queued_mode = query.stale_queued.unwrap_or(StaleMode::Poked);
    let paused_mode = query.stale_paused.unwrap_or(StaleMode::Never);
    let cutoff = Utc::now() - Duration::days(days);
    let last_touched = |node: &NodeResponse| node.last_poked_at.unwrap_or(node.created_at);
    let eligible = |node: &NodeResponse| {
        if node.archived {
            return false;
        }
        let mode = match node.status {
            Some(NodeStatus::Active) => return true,
            Some(NodeStatus::Queued) => queued_mode,
            Some(NodeStatus::Paused) => paused_mode,
            _ => return false,
        };
        match mode {
            StaleMode::Always => true,
            StaleMode::Poked => node.last_poked_at.is_some(),
            StaleMode::Never => false,
        }
    };
    let mut stale: Vec<NodeResponse> = all_nodes
        .iter()
        .filter(|node| eligible(node))
        .filter(|node| last_touched(node) < cutoff)
        .cloned()
        .collect();
    stale.sort_by_key(last_touched);
    stale
}
