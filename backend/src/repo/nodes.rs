use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::node::{
    CreateNodeRequest, NodeFocus, NodeKind, NodeListQuery, NodeResponse, NodeStatus, NodeView,
    Progress, UpdateNodeRequest,
};
use crate::repo::{checklist, edges, node_topics, pokes};

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

    let row = sqlx::query_as!(
        NodeRow,
        r#"
        -- Same auto-stamping as update_node: a node created straight into
        -- `active`/`done` gets started_at/completed_at, not just one that
        -- transitions there later.
        INSERT INTO nodes (
            user_id, kind, status, focus, title, progress_current, progress_total, color, notes,
            started_at, completed_at, progress_unit
        )
        VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9,
            CASE WHEN $3 = 'active'::node_status THEN now() END,
            CASE WHEN $3 = 'done'::node_status THEN now() END,
            $10
        )
        RETURNING
            id, kind AS "kind: NodeKind", status AS "status: NodeStatus", focus AS "focus: NodeFocus",
            title, progress_current, progress_total, progress_unit, color, notes,
            created_at, updated_at, started_at, completed_at,
            canvas_x, canvas_y,
            ARRAY[]::uuid[] AS "topic_ids!: Vec<Uuid>",
            false AS "blocked!",
            0::bigint AS "container_total!",
            0::bigint AS "container_done!",
            0::bigint AS "checklist_total!",
            0::bigint AS "checklist_done!",
            NULL::timestamptz AS last_poked_at
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
    .fetch_one(pool)
    .await?;

    Ok(row.into())
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
            n.color, n.notes, n.created_at, n.updated_at, n.started_at, n.completed_at,
            n.canvas_x, n.canvas_y,
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
/// implies. `backlog` = unpromoted ideas: kind=idea AND status=idea, since
/// promoting to project/study changes `kind` - an idea still at
/// kind='idea'/status='idea' has never been promoted. `archived` =
/// status=archived. `all`/`None` = no extra constraint.
fn view_predicates(view: Option<NodeView>) -> (Option<NodeKind>, Option<NodeStatus>) {
    match view {
        Some(NodeView::Backlog) => (Some(NodeKind::Idea), Some(NodeStatus::Idea)),
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
            n.color, n.notes, n.created_at, n.updated_at, n.started_at, n.completed_at,
            n.canvas_x, n.canvas_y,
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
    let started_at_set = request.started_at.is_some();
    let started_at = request.started_at.flatten();
    let completed_at_set = request.completed_at.is_some();
    let completed_at = request.completed_at.flatten();
    let canvas_x_set = request.canvas_x.is_some();
    let canvas_x = request.canvas_x.flatten();
    let canvas_y_set = request.canvas_y.is_some();
    let canvas_y = request.canvas_y.flatten();
    let progress_unit_set = request.progress_unit.is_some();
    let progress_unit = request.progress_unit.clone().flatten();

    let row = sqlx::query!(
        r#"
        UPDATE nodes SET
            kind = COALESCE($3, kind),
            status = COALESCE($4, status),
            focus = COALESCE($5, focus),
            title = COALESCE($6, title),
            progress_current = CASE WHEN $7 THEN $8 ELSE progress_current END,
            progress_total = CASE WHEN $9 THEN $10 ELSE progress_total END,
            color = CASE WHEN $11 THEN $12 ELSE color END,
            notes = CASE WHEN $13 THEN $14 ELSE notes END,
            -- An explicit client value always wins ($15/$17). Otherwise,
            -- the server tracks these itself: `started_at` is stamped the
            -- first time status moves to `active` (never overwritten
            -- again), and `completed_at` the first time it moves to
            -- `done` - cleared automatically if status later moves away
            -- from `done`, since it's no longer true.
            started_at = CASE
                WHEN $15 THEN $16
                WHEN COALESCE($4, status) = 'active' AND started_at IS NULL THEN now()
                ELSE started_at
            END,
            completed_at = CASE
                WHEN $17 THEN $18
                WHEN COALESCE($4, status) = 'done' AND completed_at IS NULL THEN now()
                WHEN COALESCE($4, status) <> 'done' THEN NULL
                ELSE completed_at
            END,
            canvas_x = CASE WHEN $19 THEN $20 ELSE canvas_x END,
            canvas_y = CASE WHEN $21 THEN $22 ELSE canvas_y END,
            progress_unit = CASE WHEN $23 THEN $24 ELSE progress_unit END,
            updated_at = now()
        WHERE user_id = $1 AND id = $2
        RETURNING
            id, kind AS "kind: NodeKind", status AS "status: NodeStatus", focus AS "focus: NodeFocus",
            title, progress_current, progress_total, progress_unit, color, notes,
            created_at, updated_at, started_at, completed_at, canvas_x, canvas_y
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
        started_at_set,
        started_at,
        completed_at_set,
        completed_at,
        canvas_x_set,
        canvas_x,
        canvas_y_set,
        canvas_y,
        progress_unit_set,
        progress_unit,
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };
    let topic_ids = node_topics::topic_ids_for_node(user_id, pool, row.id).await?;
    let derived = edges::derived_state(user_id, pool, row.id).await?;
    let last_poked_at = pokes::last_poked_at(user_id, pool, row.id).await?;
    let checklist_progress = checklist::progress_for_node(user_id, pool, row.id).await?;

    Ok(Some(NodeResponse {
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
        topic_ids,
        blocked: derived.blocked,
        container_progress: derived.container_progress,
        checklist_progress,
        last_poked_at,
    }))
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
