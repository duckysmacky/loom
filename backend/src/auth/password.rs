use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

/// A fixed Argon2id hash of no known password - `verify_password` against
/// this when the email lookup misses, so an unknown-email login pays the
/// same Argon2 cost as a wrong-password one. Without this, login timing
/// leaks which emails have accounts (hash+compare skipped entirely on a
/// miss).
const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$ZXhwwFsRrQuI4hRupNv4NA$pBcKDxf2vxbwgvbMGxFx5SmD+m7+I49aFfeSIYthUpU";

/// Hashes a password with Argon2id (random salt, generated internally).
/// The salt is embedded in the returned PHC string, so no separate salt
/// column is needed.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    Ok(Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string())
}

/// Returns `true` if `password` matches the given Argon2id PHC hash string.
pub fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

/// `verify_password` against a fixed dummy hash - same Argon2 cost as a
/// real verification, always fails. Used so a login against an unknown
/// email takes the same time as one against a known email with the wrong
/// password.
pub fn verify_password_dummy(password: &str) {
    verify_password(password, DUMMY_PASSWORD_HASH);
}

/// Argon2id is deliberately slow (tens of milliseconds) - running it
/// inline on a tokio worker thread blocks that worker from servicing any
/// other request for the duration. `spawn_blocking` moves it to the
/// blocking thread pool instead.
pub async fn hash_password_blocking(
    password: String,
) -> Result<String, argon2::password_hash::Error> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .expect("hash_password panicked")
}

pub async fn verify_password_blocking(password: String, hash: String) -> bool {
    tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .expect("verify_password panicked")
}

pub async fn verify_password_dummy_blocking(password: String) {
    tokio::task::spawn_blocking(move || verify_password_dummy(&password))
        .await
        .expect("verify_password_dummy panicked")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_roundtrips() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &hash));
        assert!(!verify_password("wrong password", &hash));
    }

    #[test]
    fn verify_rejects_garbage_hash() {
        assert!(!verify_password("anything", "not a phc string"));
    }
}
