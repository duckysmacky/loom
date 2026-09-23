use sqlx::PgPool;
use uuid::Uuid;

use crate::models::board::CanvasResponse;
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
