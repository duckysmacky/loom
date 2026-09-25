//! The MCP tools. Each one is a thin wrapper over the REST handler that
//! does the same job - called as a plain async fn - so validation, kind
//! capability checks and error messages stay in exactly one place.

use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use axum::http::request::Parts;
use rmcp::handler::server::tool::Extension;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::handlers::error::ApiError;
use crate::handlers::extract::{ApiJson, ApiPath, ApiQuery};
use crate::handlers::{board, checklist, dashboard, edges, nodes, pokes, topics};
use crate::middleware::auth_user::AuthUser;
use crate::models::checklist::{CreateChecklistItemRequest, UpdateChecklistItemRequest};
use crate::models::edge::{CreateEdgeRequest, EdgeKind, EdgeListQuery};
use crate::models::node::{
    AttachTopicRequest, CreateNodeRequest, NodeKind, NodeListQuery, UpdateNodeRequest,
};
use crate::models::topic::{CreateTopicRequest, UpdateTopicRequest};
use crate::state::AppState;

const INSTRUCTIONS: &str = "\
Loom is the user's personal graph of projects, studies and ideas.

Node kinds: `idea` (uncommitted capture - no progress, no checklist), `project` \
(the only kind with a checklist), `study` (courses/books/topics - the only kind \
with a progress counter: progress_current/progress_total/progress_unit), `path` \
(the only container - other nodes join it with a `part_of` edge; paths can nest).

Status: idea (= backlog) -> queued -> active -> paused -> done, or archived. \
Focus tier (primary/secondary/background) is orthogonal to status and drives the \
dashboard. Changing a node's kind (\"promoting\") keeps its id and edges but drops \
data the new kind can't hold (leaving project deletes its checklist, leaving study \
clears progress, leaving path releases its children).

Edges always point from the node being described: `requires` (from depends on \
to - `from` is blocked until `to` is done; cycles rejected), `part_of` (from is a \
child of the path `to`; one path per node; cycles rejected), `related` (soft link).

Derived, read-only fields: blocked, container_progress (done/total of a path's \
children), checklist_progress, last_poked_at. A poke is an \"I worked on this\" \
log entry; nodes active/queued with no poke for 14 days are stale.

Read before you write: call loom_get_graph or loom_list_nodes to find existing \
nodes and topics instead of creating duplicates. Use loom_create_subgraph to \
build several connected nodes at once (e.g. a learning path). Null fields are \
omitted from results.";

/// Keys that only matter to the web canvas - noise for an agent.
const LAYOUT_KEYS: [&str; 5] = [
    "canvas_x",
    "canvas_y",
    "canvas_width",
    "canvas_height",
    "sort_order",
];

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NodeIdParams {
    pub node_id: Uuid,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EdgeIdParams {
    pub edge_id: Uuid,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChecklistItemIdParams {
    pub item_id: Uuid,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TopicIdParams {
    pub topic_id: Uuid,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NodeTopicParams {
    pub node_id: Uuid,
    pub topic_id: Uuid,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListNodesParams {
    #[serde(flatten)]
    pub filters: NodeListQuery,
    /// Case-insensitive substring to match against node titles.
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateNodeParams {
    #[serde(flatten)]
    pub node: CreateNodeRequest,
    /// Checklist item titles to add, in order. Projects only.
    #[serde(default)]
    pub checklist: Vec<String>,
    /// Existing topic ids to tag the node with (see loom_list_topics).
    #[serde(default)]
    pub topic_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateNodeParams {
    pub node_id: Uuid,
    #[serde(flatten)]
    pub changes: UpdateNodeRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AddChecklistItemsParams {
    /// Must be a project.
    pub node_id: Uuid,
    /// Item titles, appended in this order.
    pub titles: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateChecklistItemParams {
    pub item_id: Uuid,
    #[serde(flatten)]
    pub changes: UpdateChecklistItemRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateTopicParams {
    pub topic_id: Uuid,
    #[serde(flatten)]
    pub changes: UpdateTopicRequest,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubgraphNode {
    /// A local name for this node, used by `edges` in the same call.
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(flatten)]
    pub node: CreateNodeParams,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubgraphEdge {
    /// A `ref` from `nodes`, or the id of an existing node.
    pub from: String,
    /// A `ref` from `nodes`, or the id of an existing node.
    pub to: String,
    pub kind: EdgeKind,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateSubgraphParams {
    /// Nodes to create, in order.
    pub nodes: Vec<SubgraphNode>,
    /// Edges to create once every node exists.
    #[serde(default)]
    pub edges: Vec<SubgraphEdge>,
}

#[derive(Clone)]
pub struct LoomServer {
    state: AppState,
}

#[tool_router]
impl LoomServer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    fn app(&self) -> State<AppState> {
        State(self.state.clone())
    }

    /// Dashboard summary: counts by status/kind, blocked count, primary-focus
    /// nodes, stale nodes, recent backlog and paths.
    #[tool(annotations(title = "Get overview", read_only_hint = true))]
    async fn loom_get_overview(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            dashboard::get(self.app(), user)
                .await
                .map(|Json(body)| body),
        )
    }

    /// The whole graph: every node and every edge. Best first call before
    /// editing, to find existing nodes, paths and topics.
    #[tool(annotations(title = "Get graph", read_only_hint = true))]
    async fn loom_get_graph(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        let graph = board::canvas(self.app(), user).await;
        let topics = topics::list(self.app(), user).await;
        respond(graph.and_then(|Json(graph)| {
            let Json(topics) = topics?;
            Ok(json!({ "nodes": graph.nodes, "edges": graph.edges, "topics": topics }))
        }))
    }

    /// Lists nodes, newest first, optionally filtered by kind/status/focus,
    /// a view preset (`backlog` = status idea, `archived`) and a title search.
    #[tool(annotations(title = "List nodes", read_only_hint = true))]
    async fn loom_list_nodes(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<ListNodesParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        let search = params.search.map(|search| search.to_lowercase());
        respond(
            nodes::list(self.app(), user, ApiQuery(params.filters))
                .await
                .map(|Json(mut found)| {
                    if let Some(search) = &search {
                        found.retain(|node| node.title.to_lowercase().contains(search));
                    }
                    json!({ "nodes": found })
                }),
        )
    }

    /// One node in full: its fields, outgoing and incoming edges, checklist
    /// (projects) and the 10 most recent pokes.
    #[tool(annotations(title = "Get node", read_only_hint = true))]
    async fn loom_get_node(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(NodeIdParams { node_id }): Parameters<NodeIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(self.node_details(user, node_id).await)
    }

    /// Creates one node, optionally with checklist items (projects only) and
    /// topics. To create several connected nodes use loom_create_subgraph.
    #[tool(annotations(
        title = "Create node",
        read_only_hint = false,
        destructive_hint = false
    ))]
    async fn loom_create_node(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<CreateNodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(self.create_node(user, params).await)
    }

    /// Updates a node. Send only the fields to change; `null` clears a
    /// nullable field. Changing `kind` promotes it in place (same
    /// id and edges) but drops data the new kind can't hold: leaving
    /// project deletes its checklist, leaving study clears progress,
    /// leaving path releases its children.
    #[tool(annotations(title = "Update node", read_only_hint = false, destructive_hint = true))]
    async fn loom_update_node(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<UpdateNodeParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            nodes::update(
                self.app(),
                user,
                ApiPath(params.node_id),
                ApiJson(params.changes),
            )
            .await
            .map(|Json(node)| node),
        )
    }

    /// Permanently deletes a node with its edges, checklist and pokes.
    #[tool(annotations(title = "Delete node", read_only_hint = false, destructive_hint = true))]
    async fn loom_delete_node(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(NodeIdParams { node_id }): Parameters<NodeIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            nodes::delete(self.app(), user, ApiPath(node_id))
                .await
                .map(|_| json!({ "deleted": node_id })),
        )
    }

    /// Logs a poke ("I worked on this") on a node, resetting its staleness.
    #[tool(annotations(title = "Poke node", read_only_hint = false, destructive_hint = false))]
    async fn loom_poke_node(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(NodeIdParams { node_id }): Parameters<NodeIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            pokes::create(self.app(), user, ApiPath(node_id))
                .await
                .map(|(_, Json(poke))| poke),
        )
    }

    /// Connects two nodes. `requires`: `from` depends on `to` (e.g. lesson 2
    /// requires lesson 1). `part_of`: `from` joins the path `to`. `related`:
    /// a soft link.
    #[tool(annotations(
        title = "Create edge",
        read_only_hint = false,
        destructive_hint = false
    ))]
    async fn loom_create_edge(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(request): Parameters<CreateEdgeRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            edges::create(self.app(), user, ApiJson(request))
                .await
                .map(|(_, Json(edge))| edge),
        )
    }

    /// Deletes an edge by its id (see loom_get_node for a node's edges).
    #[tool(annotations(title = "Delete edge", read_only_hint = false, destructive_hint = true))]
    async fn loom_delete_edge(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(EdgeIdParams { edge_id }): Parameters<EdgeIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            edges::delete(self.app(), user, ApiPath(edge_id))
                .await
                .map(|_| json!({ "deleted": edge_id })),
        )
    }

    /// Appends checklist items to a project.
    #[tool(annotations(
        title = "Add checklist items",
        read_only_hint = false,
        destructive_hint = false
    ))]
    async fn loom_add_checklist_items(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<AddChecklistItemsParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            self.add_checklist_items(user, params.node_id, params.titles)
                .await
                .map(|items| json!({ "items": items })),
        )
    }

    /// Renames a checklist item or ticks it done/undone.
    #[tool(annotations(
        title = "Update checklist item",
        read_only_hint = false,
        destructive_hint = false
    ))]
    async fn loom_update_checklist_item(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<UpdateChecklistItemParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            checklist::update(
                self.app(),
                user,
                ApiPath(params.item_id),
                ApiJson(params.changes),
            )
            .await
            .map(|Json(item)| item),
        )
    }

    /// Deletes a checklist item.
    #[tool(annotations(
        title = "Delete checklist item",
        read_only_hint = false,
        destructive_hint = true
    ))]
    async fn loom_delete_checklist_item(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(ChecklistItemIdParams { item_id }): Parameters<ChecklistItemIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            checklist::delete(self.app(), user, ApiPath(item_id))
                .await
                .map(|_| json!({ "deleted": item_id })),
        )
    }

    /// Lists every topic (free-form tags for filtering and grouping).
    #[tool(annotations(title = "List topics", read_only_hint = true))]
    async fn loom_list_topics(
        &self,
        Extension(parts): Extension<Parts>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            topics::list(self.app(), user)
                .await
                .map(|Json(found)| json!({ "topics": found })),
        )
    }

    /// Creates a topic. Names are unique - reuse an existing topic when one fits.
    #[tool(annotations(
        title = "Create topic",
        read_only_hint = false,
        destructive_hint = false
    ))]
    async fn loom_create_topic(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(request): Parameters<CreateTopicRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            topics::create(self.app(), user, ApiJson(request))
                .await
                .map(|(_, Json(topic))| topic),
        )
    }

    /// Renames or recolors a topic.
    #[tool(annotations(
        title = "Update topic",
        read_only_hint = false,
        destructive_hint = false
    ))]
    async fn loom_update_topic(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<UpdateTopicParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            topics::update(
                self.app(),
                user,
                ApiPath(params.topic_id),
                ApiJson(params.changes),
            )
            .await
            .map(|Json(topic)| topic),
        )
    }

    /// Deletes a topic and untags every node that had it.
    #[tool(annotations(
        title = "Delete topic",
        read_only_hint = false,
        destructive_hint = true
    ))]
    async fn loom_delete_topic(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(TopicIdParams { topic_id }): Parameters<TopicIdParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            topics::delete(self.app(), user, ApiPath(topic_id))
                .await
                .map(|_| json!({ "deleted": topic_id })),
        )
    }

    /// Tags a node with a topic.
    #[tool(annotations(
        title = "Attach topic",
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = true
    ))]
    async fn loom_attach_topic(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(NodeTopicParams { node_id, topic_id }): Parameters<NodeTopicParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            nodes::attach_topic(
                self.app(),
                user,
                ApiPath(node_id),
                ApiJson(AttachTopicRequest { topic_id }),
            )
            .await
            .map(|_| json!({ "node_id": node_id, "topic_id": topic_id })),
        )
    }

    /// Removes a topic from a node.
    #[tool(annotations(
        title = "Detach topic",
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = true
    ))]
    async fn loom_detach_topic(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(NodeTopicParams { node_id, topic_id }): Parameters<NodeTopicParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        respond(
            nodes::detach_topic(self.app(), user, ApiPath((node_id, topic_id)))
                .await
                .map(|_| json!({ "node_id": node_id, "topic_id": topic_id })),
        )
    }

    /// Creates several nodes and the edges between them in one call - e.g.
    /// a learning path: one `path` node, its `study`/`project` steps each
    /// `part_of` the path, and `requires` edges for the order (step 2
    /// requires step 1). Nodes get a local `ref`; edges connect refs or
    /// existing node ids. All or nothing: on any error everything created
    /// by this call is deleted again. Returns the ref -> id map.
    #[tool(annotations(
        title = "Create subgraph",
        read_only_hint = false,
        destructive_hint = false
    ))]
    async fn loom_create_subgraph(
        &self,
        Extension(parts): Extension<Parts>,
        Parameters(params): Parameters<CreateSubgraphParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = auth(&parts)?;
        let mut created_nodes = Vec::new();
        let mut created_edges = Vec::new();
        let outcome = self
            .create_subgraph(user, params, &mut created_nodes, &mut created_edges)
            .await;
        if outcome.is_err() {
            // ponytail: compensating deletes, not one DB transaction - the
            // handlers own their transactions. A crash mid-call can leave
            // partial nodes behind.
            for edge_id in created_edges {
                let _ = edges::delete(self.app(), user, ApiPath(edge_id)).await;
            }
            for node_id in created_nodes {
                let _ = nodes::delete(self.app(), user, ApiPath(node_id)).await;
            }
        }
        Ok(match outcome {
            Ok(value) => structured(value)?,
            Err(message) => CallToolResult::error(vec![ContentBlock::text(format!(
                "{message}. Nothing was created."
            ))]),
        })
    }
}

impl LoomServer {
    async fn node_details(&self, user: AuthUser, node_id: Uuid) -> Result<Value, ApiError> {
        let Json(node) = nodes::get(self.app(), user, ApiPath(node_id)).await?;
        let edge_query = |from, to| EdgeListQuery {
            from_node_id: from,
            to_node_id: to,
            kind: None,
        };
        let Json(outgoing) =
            edges::list(self.app(), user, ApiQuery(edge_query(Some(node_id), None))).await?;
        let Json(incoming) =
            edges::list(self.app(), user, ApiQuery(edge_query(None, Some(node_id)))).await?;
        let items = if node.kind == NodeKind::Project {
            let Json(items) = checklist::list(self.app(), user, ApiPath(node_id)).await?;
            Some(items)
        } else {
            None
        };
        let Json(mut recent_pokes) = pokes::list(self.app(), user, ApiPath(node_id)).await?;
        recent_pokes.truncate(10);
        Ok(json!({
            "node": node,
            "outgoing_edges": outgoing,
            "incoming_edges": incoming,
            "checklist": items,
            "recent_pokes": recent_pokes,
        }))
    }

    /// Creates the node, then its checklist and topics; if either of those
    /// fails the node is deleted again so the call is all-or-nothing.
    async fn create_node(
        &self,
        user: AuthUser,
        params: CreateNodeParams,
    ) -> Result<Value, ApiError> {
        if !params.checklist.is_empty() && params.node.kind != NodeKind::Project {
            return Err(ApiError::InvalidInput("only projects have a checklist"));
        }
        let (_, Json(node)) = nodes::create(self.app(), user, ApiJson(params.node)).await?;
        match self
            .decorate_node(user, node.id, params.checklist, params.topic_ids)
            .await
        {
            Ok(node) => Ok(node),
            Err(error) => {
                let _ = nodes::delete(self.app(), user, ApiPath(node.id)).await;
                Err(error)
            }
        }
    }

    async fn decorate_node(
        &self,
        user: AuthUser,
        node_id: Uuid,
        checklist_titles: Vec<String>,
        topic_ids: Vec<Uuid>,
    ) -> Result<Value, ApiError> {
        let items = self
            .add_checklist_items(user, node_id, checklist_titles)
            .await?;
        for topic_id in topic_ids {
            nodes::attach_topic(
                self.app(),
                user,
                ApiPath(node_id),
                ApiJson(AttachTopicRequest { topic_id }),
            )
            .await?;
        }
        let Json(node) = nodes::get(self.app(), user, ApiPath(node_id)).await?;
        let mut value = to_value(node)?;
        if !items.is_empty() {
            value["checklist"] = to_value(items)?;
        }
        Ok(value)
    }

    async fn add_checklist_items(
        &self,
        user: AuthUser,
        node_id: Uuid,
        titles: Vec<String>,
    ) -> Result<Vec<crate::models::checklist::ChecklistItemResponse>, ApiError> {
        let mut items = Vec::with_capacity(titles.len());
        for title in titles {
            let (_, Json(item)) = checklist::create(
                self.app(),
                user,
                ApiPath(node_id),
                ApiJson(CreateChecklistItemRequest { title }),
            )
            .await?;
            items.push(item);
        }
        Ok(items)
    }

    async fn create_subgraph(
        &self,
        user: AuthUser,
        params: CreateSubgraphParams,
        created_nodes: &mut Vec<Uuid>,
        created_edges: &mut Vec<Uuid>,
    ) -> Result<Value, String> {
        let mut ids = HashMap::new();
        for (index, spec) in params.nodes.into_iter().enumerate() {
            let at = format!("nodes[{index}] (ref \"{}\")", spec.reference);
            if ids.contains_key(&spec.reference) {
                return Err(format!("{at}: duplicate ref"));
            }
            let node = self
                .create_node(user, spec.node)
                .await
                .map_err(|error| format!("{at}: {}", error_text(error)))?;
            let node_id = node["id"]
                .as_str()
                .and_then(|id| id.parse::<Uuid>().ok())
                .ok_or_else(|| format!("{at}: created node has no id"))?;
            created_nodes.push(node_id);
            ids.insert(spec.reference, node_id);
        }

        let resolve = |endpoint: &str| {
            ids.get(endpoint)
                .copied()
                .or_else(|| endpoint.parse::<Uuid>().ok())
        };
        for (index, spec) in params.edges.into_iter().enumerate() {
            let at = format!("edges[{index}] ({} -> {})", spec.from, spec.to);
            let (Some(from_node_id), Some(to_node_id)) = (resolve(&spec.from), resolve(&spec.to))
            else {
                return Err(format!(
                    "{at}: each end must be a ref from nodes or an existing node id"
                ));
            };
            let request = CreateEdgeRequest {
                from_node_id,
                to_node_id,
                kind: spec.kind,
            };
            let (_, Json(edge)) = edges::create(self.app(), user, ApiJson(request))
                .await
                .map_err(|error| format!("{at}: {}", error_text(error)))?;
            created_edges.push(edge.id);
        }

        Ok(
            json!({ "ids": ids, "nodes_created": created_nodes.len(), "edges_created": created_edges.len() }),
        )
    }
}

#[tool_handler]
impl ServerHandler for LoomServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("loom", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}

/// The caller's identity, put on the request by `mcp::authenticate` before
/// it ever reaches the MCP service.
fn auth(parts: &Parts) -> Result<AuthUser, ErrorData> {
    parts
        .extensions
        .get::<AuthUser>()
        .copied()
        .ok_or_else(|| ErrorData::internal_error("request was not authenticated", None))
}

fn error_text(error: ApiError) -> String {
    let (status, message) = error.status_and_message();
    format!("{message} (HTTP {})", status.as_u16())
}

fn to_value(value: impl Serialize) -> Result<Value, ApiError> {
    serde_json::to_value(value).map_err(|error| ApiError::Internal(error.into()))
}

/// Tool-level errors (bad input, not found, conflicts) go back as a normal
/// result with `isError`, so the agent sees the message and can correct
/// itself - a JSON-RPC error would reach the model as an opaque failure.
fn respond<T: Serialize>(result: Result<T, ApiError>) -> Result<CallToolResult, ErrorData> {
    match result.and_then(to_value) {
        Ok(value) => structured(value),
        Err(error) => Ok(CallToolResult::error(vec![ContentBlock::text(error_text(
            error,
        ))])),
    }
}

fn structured(mut value: Value) -> Result<CallToolResult, ErrorData> {
    compact(&mut value);
    if !value.is_object() {
        return Err(ErrorData::internal_error(
            "tool result must be an object",
            None,
        ));
    }
    Ok(CallToolResult::structured(value))
}

/// Drops nulls and canvas-layout keys, recursively - agents read every
/// token of a result, and a large graph is mostly both.
fn compact(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|key, field| !field.is_null() && !LAYOUT_KEYS.contains(&key.as_str()));
            map.values_mut().for_each(compact);
        }
        Value::Array(items) => items.iter_mut().for_each(compact),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_drops_nulls_and_layout_keys_recursively() {
        let mut value = json!({
            "node": { "id": "a", "notes": null, "canvas_x": 1.0, "sort_order": 2 },
            "edges": [{ "id": "e", "kind": "requires", "extra": null }],
        });
        compact(&mut value);
        assert_eq!(
            value,
            json!({ "node": { "id": "a" }, "edges": [{ "id": "e", "kind": "requires" }] })
        );
    }
}
