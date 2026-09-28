use sqlx::PgPool;
use uuid::Uuid;

use crate::models::dashboard::{DashboardCounts, DashboardResponse, KindCounts, StatusCounts};
use crate::models::node::{NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus};
use crate::repo::nodes::{self, NodeRow};

const RECENT_IDEAS_LIMIT: usize = 5;

/// Assembles the whole dashboard. Everything except the counts and the
/// stale list is carved out of one full node listing - a single-user graph
/// is small, and it keeps the `blocked`/`container_progress` derivation in
/// the one query that already computes it.
pub async fn get_dashboard(user_id: Uuid, pool: &PgPool) -> Result<DashboardResponse, sqlx::Error> {
    let mut counts = get_counts(user_id, pool).await?;
    let stale = get_stale(user_id, pool).await?;
    let all_nodes = nodes::list_nodes(user_id, pool, &NodeListQuery::default()).await?;

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

    // Everything being worked on (a path is active when something inside it
    // is): primary tier first, then secondary, background; unblocked before
    // blocked within a tier.
    let mut active: Vec<NodeResponse> = all_nodes
        .iter()
        .filter(|node| node.status == Some(NodeStatus::Active))
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

pub async fn get_counts(user_id: Uuid, pool: &PgPool) -> Result<DashboardCounts, sqlx::Error> {
    struct Row {
        status: Option<NodeStatus>,
        kind: NodeKind,
        count: i64,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT status AS "status?: NodeStatus", kind AS "kind!: NodeKind", COUNT(*) AS "count!"
        FROM node_rows
        WHERE user_id = $1
        GROUP BY status, kind
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

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
    let mut total = 0i64;

    for row in rows {
        total += row.count;
        match row.status {
            Some(NodeStatus::Idea) => by_status.idea += row.count,
            Some(NodeStatus::Queued) => by_status.queued += row.count,
            Some(NodeStatus::Active) => by_status.active += row.count,
            Some(NodeStatus::Paused) => by_status.paused += row.count,
            Some(NodeStatus::Done) => by_status.done += row.count,
            Some(NodeStatus::Archived) => by_status.archived += row.count,
            // Ideas have no status.
            None => {}
        }
        match row.kind {
            NodeKind::Idea => by_kind.idea += row.count,
            NodeKind::Project => by_kind.project += row.count,
            NodeKind::Study => by_kind.study += row.count,
            NodeKind::Path => by_kind.path += row.count,
        }
    }

    Ok(DashboardCounts {
        total,
        by_status,
        by_kind,
        // Needs the derived `blocked` flag - filled in by `get_dashboard`.
        blocked: 0,
        ideas: by_kind.idea,
    })
}

/// Same `node_rows` columns as `nodes::list_nodes`, with the 14-day
/// staleness predicate as the WHERE. Only active/queued nodes are
/// eligible - done/archived/paused aren't "going stale", they're just not
/// being worked. Ordered oldest-touched-first (most urgently stale first).
pub async fn get_stale(user_id: Uuid, pool: &PgPool) -> Result<Vec<NodeResponse>, sqlx::Error> {
    let rows = sqlx::query_as!(
        NodeRow,
        r#"
        SELECT
            id AS "id!", kind AS "kind!: NodeKind", status AS "status?: NodeStatus",
            focus AS "focus?: NodeFocus", title AS "title!",
            progress_current, progress_total, progress_unit, color, notes,
            created_at AS "created_at!", updated_at AS "updated_at!", started_at, completed_at,
            canvas_x, canvas_y, canvas_width, canvas_height, sort_order,
            topic_ids AS "topic_ids!: Vec<Uuid>", blocked AS "blocked!",
            container_total AS "container_total!", container_done AS "container_done!",
            checklist_total AS "checklist_total!", checklist_done AS "checklist_done!",
            last_poked_at,
            path_progress_done AS "path_progress_done!",
            path_progress_total AS "path_progress_total!"
        FROM node_rows
        WHERE user_id = $1
          AND status IN ('active', 'queued')
          AND COALESCE(last_poked_at, created_at) < now() - interval '14 days'
        ORDER BY COALESCE(last_poked_at, created_at) ASC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(NodeResponse::from).collect())
}
