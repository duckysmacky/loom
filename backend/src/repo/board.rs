use sqlx::PgPool;
use uuid::Uuid;

use crate::models::active_period::ActivePeriodResponse;
use crate::models::board::{CanvasResponse, TimelinePokeResponse, TimelineResponse};
use crate::models::edge::EdgeListQuery;
use crate::models::node::NodeListQuery;
use crate::repo::{edges, nodes};

/// The caller's entire graph in one payload - reuses the two existing
/// unfiltered list queries rather than a bespoke query, so it doesn't
/// duplicate the blocked/container_progress/topic_ids derivation that
/// already lives in `nodes::list_nodes`.
pub async fn get_canvas(user_id: Uuid, pool: &PgPool) -> Result<CanvasResponse, sqlx::Error> {
    let nodes = nodes::list_nodes(user_id, pool, &NodeListQuery::default()).await?;
    let edges = edges::list_edges(user_id, pool, &EdgeListQuery::default()).await?;
    Ok(CanvasResponse { nodes, edges })
}

/// Every poke and active period the caller owns, across every node, in one
/// payload - the Timeline view's data source.
pub async fn get_timeline(user_id: Uuid, pool: &PgPool) -> Result<TimelineResponse, sqlx::Error> {
    let pokes = sqlx::query_as!(
        TimelinePokeResponse,
        r#"
        SELECT p.id, p.node_id, p.poked_at
        FROM pokes p
        JOIN nodes n ON n.id = p.node_id
        WHERE n.user_id = $1
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;
    let periods = sqlx::query_as!(
        ActivePeriodResponse,
        r#"
        SELECT a.id, a.node_id, a.started_at, a.ended_at
        FROM active_periods a
        JOIN nodes n ON n.id = a.node_id
        WHERE n.user_id = $1
        ORDER BY a.node_id, a.started_at
        "#,
        user_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(TimelineResponse { pokes, periods })
}
