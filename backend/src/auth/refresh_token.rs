use chrono::Duration;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const REFRESH_TOKEN_TTL: Duration = Duration::days(30);

/// Generates a new opaque refresh token: the raw value to send in the
/// cookie, and the hash of it to store in `refresh_tokens.token_hash`.
/// Only the hash ever touches the database. The raw token is 256 bits of
/// randomness (two v4 UUIDs' worth), reusing `uuid`'s RNG rather than
/// pulling in a separate rand crate for this alone.
pub fn generate() -> (String, String) {
    let raw = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let hash = hash_token(&raw);
    (raw, hash)
}

pub fn hash_token(raw: &str) -> String {
    Sha256::digest(raw.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_returns_matching_raw_and_hash() {
        let (raw, hash) = generate();
        assert_eq!(hash, hash_token(&raw));
    }

    #[test]
    fn generate_is_random() {
        let (raw_a, hash_a) = generate();
        let (raw_b, hash_b) = generate();
        assert_ne!(raw_a, raw_b);
        assert_ne!(hash_a, hash_b);
    }

    #[test]
    fn hash_token_is_deterministic() {
        assert_eq!(hash_token("same-input"), hash_token("same-input"));
    }
}
