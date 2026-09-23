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
        })
    }
}

fn require_env(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("missing required env var {key}"))
}
