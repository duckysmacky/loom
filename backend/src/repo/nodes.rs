use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::node::{
    CreateNodeRequest, NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus, NodeView,
    Progress, ReorderNodesRequest, UpdateNodeRequest,
};
use crate::repo::{active_periods, checklist, edges};

/// Query target for every node-returning query - flat fields only, since
/// `query!`/`query_as!` map one SQL column to one struct field.
/// `container_progress`'s nested shape has no single-column representation,
/// so it's carried as two flat counts here and folded into
/// `NodeResponse::container_progress` by `From<NodeRow>`.
pub(crate) struct NodeRow {
    pub(crate) id: Uuid,
    pub(crate) kind: NodeKind,
    pub(crate) status: NodeStatus,
    pub(crate) focus: NodeFocus,
    pub(crate) title: String,
    pub(crate) progress_current: Option<i32>,
    pub(crate) progress_total: Option<i32>,
    pub(crate) progress_unit: Option<String>,
    pub(crate) color: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) started_at: Option<DateTime<Utc>>,
    pub(crate) completed_at: Option<DateTime<Utc>>,
    pub(crate) canvas_x: Option<f64>,
    pub(crate) canvas_y: Option<f64>,
    pub(crate) canvas_width: Option<f64>,
    pub(crate) canvas_height: Option<f64>,
    pub(crate) sort_order: Option<i32>,
    pub(crate) topic_ids: Vec<Uuid>,
    pub(crate) blocked: bool,
    pub(crate) container_total: i64,
    pub(crate) container_done: i64,
    pub(crate) checklist_total: i64,
    pub(crate) checklist_done: i64,
    pub(crate) last_poked_at: Option<DateTime<Utc>>,
}

impl From<NodeRow> for NodeResponse {
    fn from(row: NodeRow) -> Self {
        let container_progress = (row.container_total > 0).then_some(Progress {
            done: row.container_done,
            total: row.container_total,
        });
        let checklist_progress = (row.checklist_total > 0).then_some(Progress {
            done: row.checklist_done,
            total: row.checklist_total,
        });
        NodeResponse {
            id: row.id,
            kind: row.kind,
            status: row.status,
            focus: row.focus,
            title: row.title,
            progress_current: row.progress_current,
            progress_total: row.progress_total,
            progress_unit: row.progress_unit,
            color: row.color,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
            started_at: row.started_at,
            completed_at: row.completed_at,
            canvas_x: row.canvas_x,
            canvas_y: row.canvas_y,
            canvas_width: row.canvas_width,
            canvas_height: row.canvas_height,
            sort_order: row.sort_order,
            topic_ids: row.topic_ids,
            blocked: row.blocked,
            container_progress,
            checklist_progress,
            last_poked_at: row.last_poked_at,
        }
    }
}

pub async fn create_node(
    user_id: Uuid,
    pool: &PgPool,
    request: &CreateNodeRequest,
) -> Result<NodeResponse, sqlx::Error> {
    let status = request.status.unwrap_or(NodeStatus::Idea);
    let focus = request.focus.unwrap_or(NodeFocus::Secondary);

    let mut tx = pool.begin().await?;
    let node_id = sqlx::query_scalar!(
        r#"
        INSERT INTO nodes (
            user_id, kind, status, focus, title, progress_current, progress_total, color, notes,
            progress_unit
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
        user_id,
        request.kind as NodeKind,
        status as NodeStatus,
        focus as NodeFocus,
        request.title,
        request.progress_current,
        request.progress_total,
        request.color,
        request.notes,
        request.progress_unit,
    )
    .fetch_one(&mut *tx)
    .await?;
    // A node created straight into `active` gets its first period, exactly
    // as if it had been captured as an idea and then activated.
    active_periods::sync_status_transition(
        user_id,
        &mut tx,
        node_id,
        NodeStatus::Idea,
        status,
        false,
    )
    .await?;
    tx.commit().await?;

    get_node(user_id, pool, node_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn list_nodes(
    user_id: Uuid,
    pool: &PgPool,
    filters: &NodeListQuery,
) -> Result<Vec<NodeResponse>, sqlx::Error> {
    let (view_kind, view_status) = view_predicates(filters.view);

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
          AND ($2::node_status IS NULL OR n.status = $2)
          AND ($3::node_focus IS NULL OR n.focus = $3)
          AND ($4::node_kind IS NULL OR n.kind = $4)
          AND ($5::node_kind IS NULL OR n.kind = $5)
          AND ($6::node_status IS NULL OR n.status = $6)
        GROUP BY n.id
        ORDER BY n.created_at DESC
        "#,
        user_id,
        filters.status as Option<NodeStatus>,
        filters.focus as Option<NodeFocus>,
        filters.kind as Option<NodeKind>,
        view_kind as Option<NodeKind>,
        view_status as Option<NodeStatus>,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(NodeResponse::from).collect())
}

/// Translates `view` into the effective (kind, status) constraint it
/// implies. `backlog` = everything still at status=idea, whatever its kind:
/// captured but not yet picked up (moving it to queued/active takes it out).
/// `archived` = status=archived. `all`/`None` = no extra constraint.
fn view_predicates(view: Option<NodeView>) -> (Option<NodeKind>, Option<NodeStatus>) {
    match view {
        Some(NodeView::Backlog) => (None, Some(NodeStatus::Idea)),
        Some(NodeView::Archived) => (None, Some(NodeStatus::Archived)),
        Some(NodeView::All) | None => (None, None),
    }
}

pub async fn get_node(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
) -> Result<Option<NodeResponse>, sqlx::Error> {
    let row = sqlx::query_as!(
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
        WHERE n.user_id = $1 AND n.id = $2
        GROUP BY n.id
        "#,
        user_id,
        node_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(NodeResponse::from))
}

pub async fn update_node(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
    request: &UpdateNodeRequest,
) -> Result<Option<NodeResponse>, sqlx::Error> {
    let progress_current_set = request.progress_current.is_some();
    let progress_current = request.progress_current.flatten();
    let progress_total_set = request.progress_total.is_some();
    let progress_total = request.progress_total.flatten();
    let color_set = request.color.is_some();
    let color = request.color.clone().flatten();
    let notes_set = request.notes.is_some();
    let notes = request.notes.clone().flatten();
    let canvas_x_set = request.canvas_x.is_some();
    let canvas_x = request.canvas_x.flatten();
    let canvas_y_set = request.canvas_y.is_some();
    let canvas_y = request.canvas_y.flatten();
    let canvas_width_set = request.canvas_width.is_some();
    let canvas_width = request.canvas_width.flatten();
    let canvas_height_set = request.canvas_height.is_some();
    let canvas_height = request.canvas_height.flatten();
    let progress_unit_set = request.progress_unit.is_some();
    let progress_unit = request.progress_unit.clone().flatten();

    let mut tx = pool.begin().await?;

    // The pre-update status, locked in the same transaction - the period
    // sync below needs to know which transition just happened.
    let Some(old_status) = sqlx::query_scalar!(
        r#"SELECT status AS "status: NodeStatus" FROM nodes WHERE id = $1 AND user_id = $2 FOR UPDATE"#,
        node_id,
        user_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    else {
        return Ok(None);
    };

    let row = sqlx::query!(
        r#"
        UPDATE nodes SET
            kind = COALESCE($3, kind),
            status = COALESCE($4, status),
            focus = COALESCE($5, focus),
            title = COALESCE($6, title),
            -- Tracked progress belongs to study nodes only: any other resulting
            -- kind drops it (a kind change clears data the new kind can't have).
            progress_current = CASE
                WHEN COALESCE($3, kind) <> 'study' THEN NULL
                WHEN $7 THEN $8 ELSE progress_current
            END,
            progress_total = CASE
                WHEN COALESCE($3, kind) <> 'study' THEN NULL
                WHEN $9 THEN $10 ELSE progress_total
            END,
            color = CASE WHEN $11 THEN $12 ELSE color END,
            notes = CASE WHEN $13 THEN $14 ELSE notes END,
            canvas_x = CASE WHEN $15 THEN $16 ELSE canvas_x END,
            canvas_y = CASE WHEN $17 THEN $18 ELSE canvas_y END,
            canvas_width = CASE WHEN $19 THEN $20 ELSE canvas_width END,
            canvas_height = CASE WHEN $21 THEN $22 ELSE canvas_height END,
            progress_unit = CASE
                WHEN COALESCE($3, kind) <> 'study' THEN NULL
                WHEN $23 THEN $24 ELSE progress_unit
            END,
            updated_at = now()
        WHERE user_id = $1 AND id = $2
        RETURNING kind AS "kind: NodeKind", status AS "status: NodeStatus"
        "#,
        user_id,
        node_id,
        request.kind as Option<NodeKind>,
        request.status as Option<NodeStatus>,
        request.focus as Option<NodeFocus>,
        request.title,
        progress_current_set,
        progress_current,
        progress_total_set,
        progress_total,
        color_set,
        color,
        notes_set,
        notes,
        canvas_x_set,
        canvas_x,
        canvas_y_set,
        canvas_y,
        canvas_width_set,
        canvas_width,
        canvas_height_set,
        canvas_height,
        progress_unit_set,
        progress_unit,
    )
    .fetch_one(&mut *tx)
    .await?;

    // Started/completed are the edges of the active periods: a status change
    // moves them first, then an explicit date in the request wins, as the
    // old auto-stamped columns did.
    active_periods::sync_status_transition(
        user_id,
        &mut tx,
        node_id,
        old_status,
        row.status,
        request.track_active_periods,
    )
    .await?;
    if let Some(started_at) = request.started_at {
        active_periods::set_started(user_id, &mut tx, node_id, started_at).await?;
    }
    if let Some(completed_at) = request.completed_at {
        active_periods::set_completed(user_id, &mut tx, node_id, completed_at).await?;
    }
    // Checklists belong to projects only - same kind-change rule as the
    // progress columns above, in the same transaction.
    if row.kind != NodeKind::Project {
        checklist::delete_all_for_node(user_id, &mut tx, node_id).await?;
    }
    // Only paths contain nodes: a node that stops being one lets go of them.
    if row.kind != NodeKind::Path {
        edges::release_children(user_id, &mut tx, node_id).await?;
    }
    tx.commit().await?;

    get_node(user_id, pool, node_id).await
}

/// Ranks the given nodes 1..N in list order; anything left out keeps its
/// existing `sort_order`. Fails (rolled back) if any id isn't the caller's -
/// same ownership guarantee as every other write here.
pub async fn reorder_nodes(
    user_id: Uuid,
    pool: &PgPool,
    request: &ReorderNodesRequest,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let result = sqlx::query!(
        r#"
        UPDATE nodes SET sort_order = ranked.rank
        FROM unnest($2::uuid[]) WITH ORDINALITY AS ranked(id, rank)
        WHERE nodes.id = ranked.id AND nodes.user_id = $1
        "#,
        user_id,
        &request.node_ids,
    )
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() as usize != request.node_ids.len() {
        tx.rollback().await?;
        return Ok(false);
    }
    tx.commit().await?;
    Ok(true)
}

/// Drops manual ordering for every one of the caller's nodes - "Reset order".
pub async fn clear_order(user_id: Uuid, pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE nodes SET sort_order = NULL WHERE user_id = $1",
        user_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_node(user_id: Uuid, pool: &PgPool, node_id: Uuid) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM nodes WHERE user_id = $1 AND id = $2",
        user_id,
        node_id,
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}
