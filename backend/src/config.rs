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
        })
    }
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
    fn accepts_32_byte_jwt_secret() {
        assert!(validate_jwt_secret(&"a".repeat(32)).is_ok());
    }
}
