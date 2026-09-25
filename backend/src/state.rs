use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
    pub allow_signup: bool,
    /// The public origin (no trailing slash) the MCP server and its OAuth
    /// endpoints are reachable at - `None` means MCP is switched off.
    pub mcp_public_url: Option<String>,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: &str) -> Self {
        Self::with_signup_policy(pool, jwt_secret, true)
    }

    pub fn with_signup_policy(pool: PgPool, jwt_secret: &str, allow_signup: bool) -> Self {
        Self {
            pool,
            jwt_encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
            jwt_decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
            allow_signup,
            mcp_public_url: None,
        }
    }

    /// Switches the MCP server on, served from `public_url`.
    pub fn with_mcp(mut self, public_url: &str) -> Self {
        self.mcp_public_url = Some(public_url.trim_end_matches('/').to_string());
        self
    }
}
