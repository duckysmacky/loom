use sqlx::PgPool;
use uuid::Uuid;

use crate::models::dashboard::{DashboardCounts, KindCounts, StatusCounts};
use crate::models::node::{NodeFocus, NodeKind, NodeResponse, NodeStatus};
use crate::repo::nodes::NodeRow;

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
        course: 0,
    };
    let mut total = 0i64;

    for row in rows {
        total += row.count;
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
            NodeKind::Course => by_kind.course += row.count,
        }
    }

    Ok(DashboardCounts {
        total,
        by_status,
        by_kind,
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
            n.title, n.progress_current, n.progress_total,
            n.color, n.notes, n.created_at, n.updated_at, n.started_at, n.completed_at,
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

/// Same shape again, WHERE swapped for focus=primary/status=active/
/// unblocked. The blocked check is folded into WHERE as NOT EXISTS
/// (cleaner than computing `blocked` in the SELECT list and filtering
/// after), and since WHERE already guarantees every returned row is
/// unblocked, `blocked` is hardcoded false in the SELECT list rather than
/// recomputing the same EXISTS subquery a second time.
pub async fn get_unblocked_primary(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<Vec<NodeResponse>, sqlx::Error> {
    let rows = sqlx::query_as!(
        NodeRow,
        r#"
        SELECT
            n.id, n.kind AS "kind: NodeKind", n.status AS "status: NodeStatus", n.focus AS "focus: NodeFocus",
            n.title, n.progress_current, n.progress_total,
            n.color, n.notes, n.created_at, n.updated_at, n.started_at, n.completed_at,
            COALESCE(array_agg(nt.topic_id) FILTER (WHERE nt.topic_id IS NOT NULL), '{}')
                AS "topic_ids!: Vec<Uuid>",
            false AS "blocked!",
            (SELECT COUNT(*) FROM edges pe WHERE pe.to_node_id = n.id AND pe.kind = 'part_of')
                AS "container_total!",
            (SELECT COUNT(*) FROM edges pe JOIN nodes child ON child.id = pe.from_node_id
             WHERE pe.to_node_id = n.id AND pe.kind = 'part_of' AND child.status = 'done')
                AS "container_done!",
            (SELECT MAX(poked_at) FROM pokes WHERE node_id = n.id) AS last_poked_at
        FROM nodes n
        LEFT JOIN node_topics nt ON nt.node_id = n.id
        WHERE n.user_id = $1
          AND n.focus = 'primary'
          AND n.status = 'active'
          AND NOT EXISTS (
                SELECT 1 FROM edges e
                JOIN nodes req ON req.id = e.to_node_id
                WHERE e.from_node_id = n.id AND e.kind = 'requires' AND req.status <> 'done'
              )
        GROUP BY n.id
        ORDER BY n.created_at DESC
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(NodeResponse::from).collect())
}
