use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// A personal MCP token as listed in Settings - never includes the raw
/// token, which is only shown once, on creation.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct McpTokenResponse {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateMcpTokenRequest {
    pub name: String,
}

/// The one response that carries the raw token - the client must copy it
/// now, only its hash is kept.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreatedMcpTokenResponse {
    pub token: McpTokenResponse,
    pub raw_token: String,
}

/// Whether this deployment has the MCP server switched on, and the URL
/// agents should connect to when it does.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct McpInfoResponse {
    pub enabled: bool,
    pub url: Option<String>,
}
