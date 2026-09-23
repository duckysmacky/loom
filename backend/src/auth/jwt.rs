use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ACCESS_TOKEN_TTL: Duration = Duration::minutes(15);

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

pub fn issue_access_token(
    user_id: Uuid,
    encoding_key: &EncodingKey,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        iat: now.timestamp(),
        exp: (now + ACCESS_TOKEN_TTL).timestamp(),
    };
    encode(&jsonwebtoken::Header::default(), &claims, encoding_key)
}

pub fn verify_access_token(
    token: &str,
    decoding_key: &DecodingKey,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(token, decoding_key, &Validation::default()).map(|data| data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_then_verify_roundtrips() {
        let secret = b"test-secret";
        let encoding_key = EncodingKey::from_secret(secret);
        let decoding_key = DecodingKey::from_secret(secret);
        let user_id = Uuid::new_v4();

        let token = issue_access_token(user_id, &encoding_key).unwrap();
        let claims = verify_access_token(&token, &decoding_key).unwrap();

        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn verify_rejects_token_signed_with_different_secret() {
        let encoding_key = EncodingKey::from_secret(b"secret-a");
        let decoding_key = DecodingKey::from_secret(b"secret-b");

        let token = issue_access_token(Uuid::new_v4(), &encoding_key).unwrap();

        assert!(verify_access_token(&token, &decoding_key).is_err());
    }
}
