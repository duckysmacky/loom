use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The OAuth authorization request as the client put it in the consent
/// page's URL - forwarded verbatim by the page to the backend.
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct AuthorizeRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub state: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub scope: Option<String>,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub resource: Option<String>,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct AuthorizeDecision {
    #[serde(flatten)]
    pub request: AuthorizeRequest,
    pub approve: bool,
}

/// What the consent page shows before the user decides.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AuthorizePreview {
    pub client_name: String,
    pub redirect_host: String,
}

/// Where the consent page sends the browser after a decision - the
/// client's redirect URI carrying either the code or `access_denied`.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AuthorizeRedirect {
    pub redirect_to: String,
}

/// A connector the user has authorized, as listed in Settings.
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ConnectedClientResponse {
    pub client_id: String,
    pub client_name: String,
    pub connected_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// RFC 7591 registration request - only the fields Loom acts on.
#[derive(Debug, Deserialize)]
pub struct RegisterClientRequest {
    #[serde(default)]
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub client_name: Option<String>,
}

/// RFC 6749 token request (form-encoded), for both grant types.
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub code_verifier: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
}
