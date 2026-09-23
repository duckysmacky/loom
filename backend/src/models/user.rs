use uuid::Uuid;

/// A `users` row (only the columns auth actually reads - `created_at`/
/// `updated_at` aren't needed yet). Never crosses the API boundary
/// directly - `password_hash` must never leave the backend, use
/// `models::auth::AuthUserView` for that.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
}
