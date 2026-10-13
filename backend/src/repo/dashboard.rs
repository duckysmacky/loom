use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::dashboard::{DashboardCounts, DashboardResponse, KindCounts, StatusCounts};
use crate::models::node::{NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus};
use crate::repo::nodes;

const RECENT_IDEAS_LIMIT: usize = 5;
const STALE_AFTER_DAYS: i64 = 14;

/// Assembles the whole dashboard, all carved out of one full node listing -
/// a single-user graph is small, and every extra `node_rows` read re-derives
/// every path rollup.
pub async fn get_dashboard(user_id: Uuid, pool: &PgPool) -> Result<DashboardResponse, sqlx::Error> {
    let all_nodes = nodes::list_nodes(user_id, pool, &NodeListQuery::default()).await?;
    let mut counts = count_nodes(&all_nodes);
    let stale = stale_nodes(&all_nodes);

    // Ideas (no status) aren't in play: they're off the board until promoted.
    let in_play = |node: &&NodeResponse| {
        !matches!(
            node.status,
            None | Some(NodeStatus::Done | NodeStatus::Archived)
        )
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
        .filter(|node| node.status == Some(NodeStatus::Active) && node.kind != NodeKind::Path)
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
        .filter(|node| node.kind == NodeKind::Idea)
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
        match node.status {
            Some(NodeStatus::Idea) => by_status.idea += 1,
            Some(NodeStatus::Queued) => by_status.queued += 1,
            Some(NodeStatus::Active) => by_status.active += 1,
            Some(NodeStatus::Paused) => by_status.paused += 1,
            Some(NodeStatus::Done) => by_status.done += 1,
            Some(NodeStatus::Archived) => by_status.archived += 1,
            // Ideas have no status.
            None => {}
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

/// Active/queued nodes untouched (no poke, else created) for 14+ days -
/// done/archived/paused aren't "going stale", they're just not being
/// worked. Oldest-touched first (most urgently stale first).
fn stale_nodes(all_nodes: &[NodeResponse]) -> Vec<NodeResponse> {
    let cutoff = Utc::now() - Duration::days(STALE_AFTER_DAYS);
    let last_touched = |node: &NodeResponse| node.last_poked_at.unwrap_or(node.created_at);
    let mut stale: Vec<NodeResponse> = all_nodes
        .iter()
        .filter(|node| matches!(node.status, Some(NodeStatus::Active | NodeStatus::Queued)))
        .filter(|node| last_touched(node) < cutoff)
        .cloned()
        .collect();
    stale.sort_by_key(last_touched);
    stale
}
