use axum::http::HeaderValue;
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
    /// Origins allowed to call `/api` from browser JavaScript; empty = CORS off.
    pub cors_origins: Vec<HeaderValue>,
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
            cors_origins: Vec::new(),
        }
    }

    /// Switches the MCP server on, served from `public_url`.
    pub fn with_mcp(mut self, public_url: &str) -> Self {
        self.mcp_public_url = Some(public_url.trim_end_matches('/').to_string());
        self
    }

    /// Lets browser JavaScript on these origins call `/api` (see `CORS_ORIGINS`).
    /// Origins that aren't valid header values are skipped.
    pub fn with_cors_origins(mut self, origins: &[String]) -> Self {
        self.cors_origins = origins
            .iter()
            .filter_map(|origin| origin.trim_end_matches('/').parse().ok())
            .collect();
        self
    }
}
