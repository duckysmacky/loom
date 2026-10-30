use axum::{Json, extract::State};

use crate::models::mcp::McpInfoResponse;
use crate::state::AppState;

pub async fn info(State(state): State<AppState>) -> Json<McpInfoResponse> {
    Json(McpInfoResponse {
        enabled: state.mcp_public_url.is_some(),
        url: state
            .mcp_public_url
            .as_ref()
            .map(|url| format!("{url}/mcp")),
    })
}
