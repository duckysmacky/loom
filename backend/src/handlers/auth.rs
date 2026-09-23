use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use uuid::Uuid;

use super::error::AuthError;
use crate::auth::{jwt, password, refresh_token};
use crate::middleware::auth_user::AuthUser;
use crate::models::auth::{
    AuthResponse, AuthUserView, LoginRequest, RefreshResponse, SignupRequest,
};
use crate::repo;
use crate::state::AppState;

const REFRESH_COOKIE_NAME: &str = "refresh_token";
const REFRESH_COOKIE_PATH: &str = "/api/auth";

pub async fn signup(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<SignupRequest>,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), AuthError> {
    validate_credentials(&body.email, &body.password)?;

    let password_hash =
        password::hash_password(&body.password).map_err(|e| AuthError::Internal(e.into()))?;

    let user = repo::users::create_user(&state.pool, &body.email, &password_hash)
        .await
        .map_err(map_create_user_error)?;

    let (access_token, jar) = issue_tokens(&state, jar, user.id).await?;

    Ok((
        StatusCode::CREATED,
        jar,
        Json(AuthResponse {
            access_token,
            user: AuthUserView {
                id: user.id,
                email: user.email,
            },
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), AuthError> {
    let user = repo::users::find_by_email(&state.pool, &body.email)
        .await?
        .ok_or(AuthError::InvalidCredentials)?;

    if !password::verify_password(&body.password, &user.password_hash) {
        return Err(AuthError::InvalidCredentials);
    }

    let (access_token, jar) = issue_tokens(&state, jar, user.id).await?;

    Ok((
        StatusCode::OK,
        jar,
        Json(AuthResponse {
            access_token,
            user: AuthUserView {
                id: user.id,
                email: user.email,
            },
        }),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<RefreshResponse>), AuthError> {
    let raw_token = jar
        .get(REFRESH_COOKIE_NAME)
        .map(|cookie| cookie.value().to_owned())
        .ok_or(AuthError::InvalidToken)?;

    let token_hash = refresh_token::hash_token(&raw_token);
    let row = repo::refresh_tokens::find_valid_by_hash(&state.pool, &token_hash)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    // Rotation: the old token is single-use, revoke it before issuing the next one.
    repo::refresh_tokens::revoke(&state.pool, row.id).await?;

    let (access_token, jar) = issue_tokens(&state, jar, row.user_id).await?;

    Ok((jar, Json(RefreshResponse { access_token })))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar), AuthError> {
    if let Some(raw_token) = jar.get(REFRESH_COOKIE_NAME).map(|c| c.value().to_owned()) {
        let token_hash = refresh_token::hash_token(&raw_token);
        if let Some(row) =
            repo::refresh_tokens::find_valid_by_hash(&state.pool, &token_hash).await?
        {
            repo::refresh_tokens::revoke(&state.pool, row.id).await?;
        }
    }

    Ok((StatusCode::NO_CONTENT, jar.remove(clear_refresh_cookie())))
}

pub async fn me(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<AuthUserView>, AuthError> {
    let user = repo::users::find_by_id(&state.pool, user_id)
        .await?
        .ok_or(AuthError::InvalidToken)?;

    Ok(Json(AuthUserView {
        id: user.id,
        email: user.email,
    }))
}

/// Issues a fresh access token and a fresh (stored) refresh token, and
/// returns the jar with the refresh cookie set.
async fn issue_tokens(
    state: &AppState,
    jar: CookieJar,
    user_id: Uuid,
) -> Result<(String, CookieJar), AuthError> {
    let access_token = jwt::issue_access_token(user_id, &state.jwt_encoding_key)
        .map_err(|e| AuthError::Internal(e.into()))?;

    let (raw_refresh_token, refresh_token_hash) = refresh_token::generate();
    let expires_at = Utc::now() + refresh_token::REFRESH_TOKEN_TTL;
    repo::refresh_tokens::insert(&state.pool, user_id, &refresh_token_hash, expires_at).await?;

    Ok((
        access_token,
        jar.add(build_refresh_cookie(raw_refresh_token)),
    ))
}

fn build_refresh_cookie(value: String) -> Cookie<'static> {
    Cookie::build((REFRESH_COOKIE_NAME, value))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path(REFRESH_COOKIE_PATH)
        .max_age(cookie::time::Duration::seconds(
            refresh_token::REFRESH_TOKEN_TTL.num_seconds(),
        ))
        .into()
}

/// A cookie with matching name/path but no value/max-age, so the jar
/// serializes it as an immediate expiry - the browser only clears a cookie
/// that matches name, path, and domain.
fn clear_refresh_cookie() -> Cookie<'static> {
    Cookie::build(REFRESH_COOKIE_NAME)
        .path(REFRESH_COOKIE_PATH)
        .into()
}

fn validate_credentials(email: &str, password: &str) -> Result<(), AuthError> {
    if !email.contains('@') || email.len() > 254 {
        return Err(AuthError::InvalidInput("invalid email"));
    }
    if password.len() < 8 {
        return Err(AuthError::InvalidInput(
            "password must be at least 8 characters",
        ));
    }
    Ok(())
}

fn map_create_user_error(error: sqlx::Error) -> AuthError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_unique_violation()
    {
        return AuthError::EmailTaken;
    }
    AuthError::Internal(error.into())
}
