use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

/// What a personal access token may do: `read` allows only reads (GET
/// requests on the REST API, read-only MCP tools), `read_write` everything
/// the account can. OAuth connector tokens are always `read_write`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, TS)]
#[ts(export)]
#[sqlx(type_name = "token_scope", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TokenScope {
    #[default]
    Read,
    ReadWrite,
}

impl TokenScope {
    pub fn allows_write(self) -> bool {
        self == Self::ReadWrite
    }
}

/// A personal access token as listed in Settings - never includes the raw
/// token, which is only shown once, on creation.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct AccessTokenResponse {
    pub id: Uuid,
    pub name: String,
    pub scope: TokenScope,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    /// `None` = never expires.
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CreateAccessTokenRequest {
    pub name: String,
    /// Defaults to `read`: a token has to be given write access on purpose.
    #[serde(default)]
    pub scope: TokenScope,
    /// Days until the token stops working; `None` = never.
    #[serde(default)]
    pub expires_in_days: Option<u32>,
}

/// The one response that carries the raw token - the client must copy it
/// now, only its hash is kept.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CreatedAccessTokenResponse {
    pub token: AccessTokenResponse,
    pub raw_token: String,
}
