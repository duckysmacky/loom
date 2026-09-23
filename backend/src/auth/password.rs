use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

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
