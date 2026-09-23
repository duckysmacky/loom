use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::User;

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id, email, password_hash
        "#,
        email,
        password_hash,
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash
        FROM users
        WHERE email = $1
        "#,
        email,
    )
    .fetch_optional(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, user_id: Uuid) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, email, password_hash
        FROM users
        WHERE id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await
}
