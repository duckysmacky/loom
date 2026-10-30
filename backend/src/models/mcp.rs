use serde::Serialize;
use ts_rs::TS;

/// Whether this deployment has the MCP server switched on, and the URL
/// agents should connect to when it does.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct McpInfoResponse {
    pub enabled: bool,
    pub url: Option<String>,
}
