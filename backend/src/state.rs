use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: &str) -> Self {
        Self {
            pool,
            jwt_encoding_key: EncodingKey::from_secret(jwt_secret.as_bytes()),
            jwt_decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
        }
    }
}
