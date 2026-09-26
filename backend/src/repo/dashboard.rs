use sqlx::PgPool;
use uuid::Uuid;

use crate::models::dashboard::{DashboardCounts, DashboardResponse, KindCounts, StatusCounts};
use crate::models::node::{NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus};
use crate::repo::nodes::{self, NodeRow};

const RECENT_BACKLOG_LIMIT: usize = 5;

/// Assembles the whole dashboard. Everything except the counts and the
/// stale list is carved out of one full node listing - a single-user graph
/// is small, and it keeps the `blocked`/`container_progress` derivation in
/// the one query that already computes it.
pub async fn get_dashboard(user_id: Uuid, pool: &PgPool) -> Result<DashboardResponse, sqlx::Error> {
    let mut counts = get_counts(user_id, pool).await?;
    let stale = get_stale(user_id, pool).await?;
    let all_nodes = nodes::list_nodes(user_id, pool, &NodeListQuery::default()).await?;

    let in_play =
        |node: &&NodeResponse| !matches!(node.status, NodeStatus::Done | NodeStatus::Archived);

    counts.blocked = all_nodes
        .iter()
        .filter(in_play)
        .filter(|node| node.blocked)
        .count() as i64;

    let mut primary: Vec<NodeResponse> = all_nodes
        .iter()
        .filter(in_play)
        .filter(|node| node.focus == NodeFocus::Primary)
        .cloned()
        .collect();
    // Stable sort - within a rank the listing's newest-first order holds.
    primary.sort_by_key(|node| match (node.status, node.blocked) {
        (NodeStatus::Active, false) => 0,
        (_, true) => 1,
        _ => 2,
    });

    let recent_backlog = all_nodes
        .iter()
        .filter(|node| node.status == NodeStatus::Idea)
        .take(RECENT_BACKLOG_LIMIT)
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
        primary,
        recent_backlog,
        paths,
    })
}

pub async fn get_counts(user_id: Uuid, pool: &PgPool) -> Result<DashboardCounts, sqlx::Error> {
    struct Row {
        status: NodeStatus,
        kind: NodeKind,
        count: i64,
    }

    let rows = sqlx::query_as!(
        Row,
        r#"
        SELECT status AS "status: NodeStatus", kind AS "kind: NodeKind", COUNT(*) AS "count!"
        FROM nodes
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
    let mut backlog = 0i64;

    for row in rows {
        total += row.count;
        // Backlog = still at status idea, whatever the kind.
        if row.status == NodeStatus::Idea {
            backlog += row.count;
        }
        match row.status {
            NodeStatus::Idea => by_status.idea += row.count,
            NodeStatus::Queued => by_status.queued += row.count,
            NodeStatus::Active => by_status.active += row.count,
            NodeStatus::Paused => by_status.paused += row.count,
            NodeStatus::Done => by_status.done += row.count,
            NodeStatus::Archived => by_status.archived += row.count,
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
        backlog,
    })
}

/// Same SELECT/JOIN shape as `nodes::list_nodes` (duplicated - `query_as!`
/// is checked against a literal string, true reuse isn't practical), swapped
/// WHERE for the 14-day staleness predicate. Only active/queued nodes are
/// eligible - done/archived/paused aren't "going stale", they're just not
/// being worked. Ordered oldest-touched-first (most urgently stale first).
pub async fn get_stale(user_id: Uuid, pool: &PgPool) -> Result<Vec<NodeResponse>, sqlx::Error> {
    let rows = sqlx::query_as!(
        NodeRow,
        r#"
        SELECT
            n.id, n.kind AS "kind: NodeKind", n.status AS "status: NodeStatus", n.focus AS "focus: NodeFocus",
            n.title, n.progress_current, n.progress_total, n.progress_unit,
            n.color, n.notes, n.created_at, n.updated_at,
            (SELECT MIN(a.started_at) FROM active_periods a WHERE a.node_id = n.id) AS started_at,
            CASE WHEN n.status = 'done' THEN (
                SELECT a.ended_at FROM active_periods a WHERE a.node_id = n.id
                ORDER BY a.started_at DESC, a.id DESC LIMIT 1
            ) END AS completed_at,
            n.canvas_x, n.canvas_y, n.canvas_width, n.canvas_height, n.sort_order,
            COALESCE(array_agg(nt.topic_id) FILTER (WHERE nt.topic_id IS NOT NULL), '{}')
                AS "topic_ids!: Vec<Uuid>",
            EXISTS (
                SELECT 1 FROM edges e
                JOIN nodes req ON req.id = e.to_node_id
                WHERE e.from_node_id = n.id AND e.kind = 'requires' AND req.status <> 'done'
            ) AS "blocked!",
            (SELECT COUNT(*) FROM edges pe WHERE pe.to_node_id = n.id AND pe.kind = 'part_of')
                AS "container_total!",
            (SELECT COUNT(*) FROM edges pe JOIN nodes child ON child.id = pe.from_node_id
             WHERE pe.to_node_id = n.id AND pe.kind = 'part_of' AND child.status = 'done')
                AS "container_done!",
            (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = n.id)
                AS "checklist_total!",
            (SELECT COUNT(*) FROM checklist_items ci WHERE ci.node_id = n.id AND ci.done)
                AS "checklist_done!",
            (SELECT MAX(poked_at) FROM pokes WHERE node_id = n.id) AS last_poked_at
        FROM nodes n
        LEFT JOIN node_topics nt ON nt.node_id = n.id
        WHERE n.user_id = $1
          AND n.status IN ('active', 'queued')
          AND COALESCE(
                (SELECT MAX(poked_at) FROM pokes WHERE node_id = n.id),
                n.created_at
              ) < now() - interval '14 days'
        GROUP BY n.id
        ORDER BY COALESCE((SELECT MAX(poked_at) FROM pokes WHERE node_id = n.id), n.created_at) ASC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(NodeResponse::from).collect())
}
