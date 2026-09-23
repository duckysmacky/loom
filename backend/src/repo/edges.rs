use sqlx::PgPool;
use uuid::Uuid;

use crate::models::edge::{CreateEdgeRequest, EdgeKind, EdgeListQuery, EdgeResponse};
use crate::models::node::{NodeKind, Progress};
use sqlx::PgConnection;

pub enum CreateEdgeOutcome {
    Created(EdgeResponse),
    NotFound,
    WouldCreateCycle,
    AlreadyExists,
    /// `part_of` pointing at a node that isn't a path.
    NotAPath,
    /// `part_of` from a node that already sits in a path.
    AlreadyInPath,
}

pub struct NodeDerivedState {
    pub blocked: bool,
    pub container_progress: Option<Progress>,
}

pub async fn create_edge(
    user_id: Uuid,
    pool: &PgPool,
    request: &CreateEdgeRequest,
) -> Result<CreateEdgeOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let owned = sqlx::query_scalar!(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM nodes WHERE id = $1 AND user_id = $3)
            AND EXISTS(SELECT 1 FROM nodes WHERE id = $2 AND user_id = $3)
            AS "owned!"
        "#,
        request.from_node_id,
        request.to_node_id,
        user_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    if !owned {
        return Ok(CreateEdgeOutcome::NotFound);
    }

    // Only paths contain nodes.
    if request.kind == EdgeKind::PartOf {
        let target_kind = sqlx::query_scalar!(
            r#"SELECT kind AS "kind: NodeKind" FROM nodes WHERE id = $1"#,
            request.to_node_id,
        )
        .fetch_one(&mut *tx)
        .await?;
        if target_kind != NodeKind::Path {
            return Ok(CreateEdgeOutcome::NotAPath);
        }
    }

    // `requires` cycles would make `blocked` unsatisfiable forever; `part_of`
    // cycles would make container progress meaningless (a container whose
    // progress derives from a child that's itself derived from it) and
    // break canvas nesting. `related` is a soft, non-hierarchical
    // association - cycles there are fine.
    if matches!(request.kind, EdgeKind::Requires | EdgeKind::PartOf) {
        // Serializes concurrent same-kind edge inserts for this user so two
        // requests can't both pass the cycle check below before either
        // commits. Per-user, not global - cheap at this app's scale.
        // ponytail: per-user advisory lock, revisit only if edge writes
        // become a hot path (they won't, this is a single-user graph).
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtext($1)::bigint)",
            user_id.to_string()
        )
        .execute(&mut *tx)
        .await?;

        // Would to_node_id already (transitively) point at from_node_id via
        // this same edge kind? If so, adding from_node_id -> to_node_id
        // would close a cycle.
        let would_cycle = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE reachable AS (
                SELECT to_node_id AS node_id FROM edges
                WHERE from_node_id = $1 AND kind = $3
                UNION
                SELECT e.to_node_id FROM edges e
                JOIN reachable r ON e.from_node_id = r.node_id
                WHERE e.kind = $3
            )
            SELECT EXISTS(SELECT 1 FROM reachable WHERE node_id = $2) AS "would_cycle!"
            "#,
            request.to_node_id,
            request.from_node_id,
            request.kind as EdgeKind,
        )
        .fetch_one(&mut *tx)
        .await?;

        if would_cycle {
            return Ok(CreateEdgeOutcome::WouldCreateCycle);
        }
    }

    let result = sqlx::query_as!(
        EdgeResponse,
        r#"
        INSERT INTO edges (from_node_id, to_node_id, kind)
        VALUES ($1, $2, $3)
        RETURNING id, from_node_id, to_node_id, kind AS "kind: EdgeKind", created_at
        "#,
        request.from_node_id,
        request.to_node_id,
        request.kind as EdgeKind,
    )
    .fetch_one(&mut *tx)
    .await;

    match result {
        Ok(edge) => {
            if edge.kind == EdgeKind::PartOf {
                reset_canvas_position(&mut tx, edge.from_node_id).await?;
            }
            tx.commit().await?;
            Ok(CreateEdgeOutcome::Created(edge))
        }
        Err(sqlx::Error::Database(db_error)) if db_error.is_unique_violation() => {
            if db_error.constraint() == Some("edges_one_path_per_node") {
                Ok(CreateEdgeOutcome::AlreadyInPath)
            } else {
                Ok(CreateEdgeOutcome::AlreadyExists)
            }
        }
        Err(error) => Err(error),
    }
}

pub async fn list_edges(
    user_id: Uuid,
    pool: &PgPool,
    filters: &EdgeListQuery,
) -> Result<Vec<EdgeResponse>, sqlx::Error> {
    sqlx::query_as!(
        EdgeResponse,
        r#"
        SELECT e.id, e.from_node_id, e.to_node_id, e.kind AS "kind: EdgeKind", e.created_at
        FROM edges e
        JOIN nodes n ON n.id = e.from_node_id
        WHERE n.user_id = $1
          AND ($2::uuid IS NULL OR e.from_node_id = $2)
          AND ($3::uuid IS NULL OR e.to_node_id = $3)
          AND ($4::edge_kind IS NULL OR e.kind = $4)
        ORDER BY e.created_at DESC
        "#,
        user_id,
        filters.from_node_id,
        filters.to_node_id,
        filters.kind as Option<EdgeKind>,
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_edge(user_id: Uuid, pool: &PgPool, edge_id: Uuid) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let deleted = sqlx::query!(
        r#"
        DELETE FROM edges e
        USING nodes n
        WHERE e.id = $2 AND e.from_node_id = n.id AND n.user_id = $1
        RETURNING e.from_node_id, e.kind AS "kind: EdgeKind"
        "#,
        user_id,
        edge_id,
    )
    .fetch_optional(&mut *tx)
    .await?;

    let Some(deleted) = deleted else {
        return Ok(false);
    };
    if deleted.kind == EdgeKind::PartOf {
        reset_canvas_position(&mut tx, deleted.from_node_id).await?;
    }
    tx.commit().await?;
    Ok(true)
}

/// Canvas positions are relative to the node's path, so joining or leaving
/// one invalidates the saved position; the canvas re-lays it out.
async fn reset_canvas_position(
    connection: &mut PgConnection,
    node_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE nodes SET canvas_x = NULL, canvas_y = NULL WHERE id = $1",
        node_id,
    )
    .execute(connection)
    .await?;
    Ok(())
}

/// Releases every node contained in `path_id` (used when it stops being a
/// path): drops their `part_of` edges and resets their canvas positions.
pub async fn release_children(
    user_id: Uuid,
    connection: &mut PgConnection,
    path_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        WITH released AS (
            DELETE FROM edges e
            USING nodes n
            WHERE e.to_node_id = $2 AND e.kind = 'part_of'
              AND n.id = e.to_node_id AND n.user_id = $1
            RETURNING e.from_node_id
        )
        UPDATE nodes SET canvas_x = NULL, canvas_y = NULL
        WHERE id IN (SELECT from_node_id FROM released)
        "#,
        user_id,
        path_id,
    )
    .execute(connection)
    .await?;
    Ok(())
}

/// The `blocked`/`container_progress` derivation for a single node - reused
/// by `repo::nodes::update_node`'s post-update follow-up, and by
/// `list_nodes`/`get_node` inline as the same three-subquery shape.
pub async fn derived_state(
    user_id: Uuid,
    pool: &PgPool,
    node_id: Uuid,
) -> Result<NodeDerivedState, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            EXISTS (
                SELECT 1 FROM edges e
                JOIN nodes req ON req.id = e.to_node_id
                WHERE e.from_node_id = n.id AND e.kind = 'requires' AND req.status <> 'done'
            ) AS "blocked!",
            (SELECT COUNT(*) FROM edges pe WHERE pe.to_node_id = n.id AND pe.kind = 'part_of')
                AS "container_total!",
            (SELECT COUNT(*) FROM edges pe JOIN nodes child ON child.id = pe.from_node_id
             WHERE pe.to_node_id = n.id AND pe.kind = 'part_of' AND child.status = 'done')
                AS "container_done!"
        FROM nodes n
        WHERE n.id = $2 AND n.user_id = $1
        "#,
        user_id,
        node_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(NodeDerivedState {
        blocked: row.blocked,
        container_progress: (row.container_total > 0).then_some(Progress {
            done: row.container_done,
            total: row.container_total,
        }),
    })
}
