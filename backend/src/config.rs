use std::env;

use anyhow::{Context, Result};

/// App configuration loaded from environment variables (`.env` in local dev).
pub struct Config {
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub bind_addr: String,
    pub jwt_secret: String,
    pub allow_signup: bool,
    /// `Some(PUBLIC_URL)` when `MCP_ENABLED=true`, `None` otherwise.
    pub mcp_public_url: Option<String>,
    /// Origins (`CORS_ORIGINS`) whose browser JavaScript may call `/api`.
    /// Empty = no CORS headers at all.
    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            db_host: require_env("DB_HOST")?,
            db_port: require_env("DB_PORT")?
                .parse()
                .context("DB_PORT must be a valid port number")?,
            db_user: require_env("DB_USER")?,
            db_password: require_env("DB_PASSWORD")?,
            db_name: require_env("DB_NAME")?,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            jwt_secret: require_jwt_secret()?,
            allow_signup: env::var("ALLOW_SIGNUP")
                .map(|v| v != "false")
                .unwrap_or(true),
            mcp_public_url: require_mcp_public_url()?,
            cors_origins: parse_cors_origins(&env::var("CORS_ORIGINS").unwrap_or_default())?,
        })
    }
}

/// The MCP server is opt-in. When on, it needs the public origin it's
/// served from: it's the OAuth issuer, the canonical resource URI agents
/// get tokens for, and the only Host the MCP transport accepts.
fn require_mcp_public_url() -> Result<Option<String>> {
    if env::var("MCP_ENABLED").map(|v| v != "true").unwrap_or(true) {
        return Ok(None);
    }
    let url = require_env("PUBLIC_URL").context("PUBLIC_URL is required when MCP_ENABLED=true")?;
    validate_public_url(&url)?;
    Ok(Some(url.trim_end_matches('/').to_string()))
}

fn validate_public_url(url: &str) -> Result<()> {
    validate_origin("PUBLIC_URL", url)
}

/// An origin is scheme + host (+ port) and nothing else.
fn validate_origin(name: &str, url: &str) -> Result<()> {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .with_context(|| {
            format!("{name} must start with https:// (or http:// for local testing)")
        })?;
    if rest.trim_end_matches('/').contains('/') {
        anyhow::bail!("{name} must be an origin with no path, e.g. https://loom.example.com");
    }
    Ok(())
}

/// `CORS_ORIGINS` is a comma-separated allowlist of exact origins. There is
/// deliberately no `*`: tokens are full-account credentials, so every site
/// allowed to hold one is named.
fn parse_cors_origins(value: &str) -> Result<Vec<String>> {
    value
        .split(',')
        .map(str::trim)
        .filter(|origin| !origin.is_empty())
        .map(|origin| {
            validate_origin("CORS_ORIGINS entries", origin)?;
            Ok(origin.trim_end_matches('/').to_string())
        })
        .collect()
}

/// HS256 signing strength is bounded by key length - a short secret (e.g.
/// the `.env.example` placeholder `change-me`, left unedited) makes access
/// tokens forgeable. `openssl rand -hex 32` (the value `.env.example`
/// recommends) is 64 bytes, so 32 is a floor well below what's expected,
/// not a tight bound.
const MIN_JWT_SECRET_LEN: usize = 32;

fn require_jwt_secret() -> Result<String> {
    let secret = require_env("JWT_SECRET")?;
    validate_jwt_secret(&secret)?;
    Ok(secret)
}

fn validate_jwt_secret(secret: &str) -> Result<()> {
    if secret.len() < MIN_JWT_SECRET_LEN {
        anyhow::bail!(
            "JWT_SECRET must be at least {MIN_JWT_SECRET_LEN} bytes - generate one with `openssl rand -hex 32`"
        );
    }
    Ok(())
}

fn require_env(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("missing required env var {key}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_short_jwt_secret() {
        assert!(validate_jwt_secret("too-short").is_err());
    }

    #[test]
    fn public_url_must_be_a_bare_origin() {
        assert!(validate_public_url("https://loom.example.com").is_ok());
        assert!(validate_public_url("https://loom.example.com/").is_ok());
        assert!(validate_public_url("http://localhost:8081").is_ok());
        assert!(validate_public_url("loom.example.com").is_err());
        assert!(validate_public_url("https://example.com/loom").is_err());
    }

    #[test]
    fn cors_origins_are_a_trimmed_list_of_bare_origins() {
        assert!(parse_cors_origins("").unwrap().is_empty());
        assert_eq!(
            parse_cors_origins(" https://a.example.com/ , http://localhost:3000,").unwrap(),
            ["https://a.example.com", "http://localhost:3000"]
        );
        assert!(parse_cors_origins("*").is_err());
        assert!(parse_cors_origins("a.example.com").is_err());
        assert!(parse_cors_origins("https://a.example.com/app").is_err());
        assert!(parse_cors_origins("https://ok.example.com,*").is_err());
    }

    #[test]
    fn accepts_32_byte_jwt_secret() {
        assert!(validate_jwt_secret(&"a".repeat(32)).is_ok());
    }
}
