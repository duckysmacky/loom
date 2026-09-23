use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::Utc;
use uuid::Uuid;

use super::error::ApiError;
use super::extract::ApiJson;
use crate::auth::{jwt, password, refresh_token};
use crate::middleware::auth_user::AuthUser;
use crate::models::auth::{
    AuthResponse, AuthUserView, ChangePasswordRequest, LoginRequest, RefreshResponse, SignupRequest,
};
use crate::repo;
use crate::state::AppState;

const REFRESH_COOKIE_NAME: &str = "refresh_token";
const REFRESH_COOKIE_PATH: &str = "/api/auth";

pub async fn signup(
    State(state): State<AppState>,
    jar: CookieJar,
    ApiJson(body): ApiJson<SignupRequest>,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), ApiError> {
    if !state.allow_signup {
        return Err(ApiError::Forbidden("signup is disabled"));
    }

    let email = normalize_email(&body.email);
    validate_credentials(&email, &body.password)?;

    let password_hash = password::hash_password_blocking(body.password)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;

    let user = repo::users::create_user(&state.pool, &email, &password_hash)
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
    ApiJson(body): ApiJson<LoginRequest>,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), ApiError> {
    let email = normalize_email(&body.email);
    let user = repo::users::find_by_email(&state.pool, &email).await?;

    // Always pay Argon2 cost, known email or not - otherwise an unknown
    // email returns near-instantly while a known one takes tens of
    // milliseconds, leaking which emails have accounts via response timing.
    let Some(user) = user else {
        password::verify_password_dummy_blocking(body.password).await;
        return Err(ApiError::InvalidCredentials);
    };

    if !password::verify_password_blocking(body.password, user.password_hash.clone()).await {
        return Err(ApiError::InvalidCredentials);
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
) -> Result<(CookieJar, Json<RefreshResponse>), ApiError> {
    let raw_token = jar
        .get(REFRESH_COOKIE_NAME)
        .map(|cookie| cookie.value().to_owned())
        .ok_or(ApiError::InvalidToken)?;

    let token_hash = refresh_token::hash_token(&raw_token);
    let user_id = match repo::refresh_tokens::consume(&state.pool, &token_hash).await? {
        repo::refresh_tokens::ConsumeOutcome::Consumed(user_id) => user_id,
        repo::refresh_tokens::ConsumeOutcome::Reused(user_id) => {
            // A rotated-out token being presented again is a theft signal -
            // kill every session for this user, not just this one.
            repo::refresh_tokens::revoke_all_for_user(&state.pool, user_id).await?;
            return Err(ApiError::InvalidToken);
        }
        repo::refresh_tokens::ConsumeOutcome::Invalid => return Err(ApiError::InvalidToken),
    };

    let (access_token, jar) = issue_tokens(&state, jar, user_id).await?;

    Ok((jar, Json(RefreshResponse { access_token })))
}

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(StatusCode, CookieJar), ApiError> {
    if let Some(raw_token) = jar.get(REFRESH_COOKIE_NAME).map(|c| c.value().to_owned()) {
        let token_hash = refresh_token::hash_token(&raw_token);
        if let repo::refresh_tokens::ConsumeOutcome::Reused(user_id) =
            repo::refresh_tokens::revoke(&state.pool, &token_hash).await?
        {
            repo::refresh_tokens::revoke_all_for_user(&state.pool, user_id).await?;
        }
    }

    Ok((StatusCode::NO_CONTENT, jar.remove(clear_refresh_cookie())))
}

pub async fn me(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
) -> Result<Json<AuthUserView>, ApiError> {
    let user = repo::users::find_by_id(&state.pool, user_id)
        .await?
        .ok_or(ApiError::InvalidToken)?;

    Ok(Json(AuthUserView {
        id: user.id,
        email: user.email,
    }))
}

/// Changes the signed-in user's password. A wrong `current_password` is a
/// 400, not a 401 - the caller's session is valid, and a 401 would make
/// clients treat it as an expired access token. Every refresh token is
/// revoked (signing out other sessions) and this session gets a fresh pair.
/// Access tokens already issued elsewhere stay valid until they expire
/// (15 min) - they're stateless JWTs.
pub async fn change_password(
    State(state): State<AppState>,
    AuthUser { user_id }: AuthUser,
    jar: CookieJar,
    ApiJson(body): ApiJson<ChangePasswordRequest>,
) -> Result<(CookieJar, Json<RefreshResponse>), ApiError> {
    validate_password(&body.new_password)?;

    let user = repo::users::find_by_id(&state.pool, user_id)
        .await?
        .ok_or(ApiError::InvalidToken)?;
    if !password::verify_password_blocking(body.current_password, user.password_hash).await {
        return Err(ApiError::InvalidInput("current password is incorrect"));
    }

    let password_hash = password::hash_password_blocking(body.new_password)
        .await
        .map_err(|e| ApiError::Internal(e.into()))?;
    repo::users::update_password_hash(&state.pool, user_id, &password_hash).await?;
    repo::refresh_tokens::revoke_all_for_user(&state.pool, user_id).await?;

    let (access_token, jar) = issue_tokens(&state, jar, user_id).await?;
    Ok((jar, Json(RefreshResponse { access_token })))
}

/// Issues a fresh access token and a fresh (stored) refresh token, and
/// returns the jar with the refresh cookie set.
async fn issue_tokens(
    state: &AppState,
    jar: CookieJar,
    user_id: Uuid,
) -> Result<(String, CookieJar), ApiError> {
    let access_token = jwt::issue_access_token(user_id, &state.jwt_encoding_key)
        .map_err(|e| ApiError::Internal(e.into()))?;

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

/// Trims and lowercases an email for storage/lookup - `Foo@x.com` and
/// `foo@x.com` are the same account. Applied before every read or write
/// that touches `users.email`.
fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

const MIN_PASSWORD_LEN: usize = 8;
/// Argon2 hashes its whole input regardless of length - an unbounded
/// password is a cheap way to burn CPU on the hashing step itself.
const MAX_PASSWORD_LEN: usize = 128;

fn validate_credentials(email: &str, password: &str) -> Result<(), ApiError> {
    if !is_valid_email(email) {
        return Err(ApiError::InvalidInput("invalid email"));
    }
    validate_password(password)
}

fn validate_password(password: &str) -> Result<(), ApiError> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(ApiError::InvalidInput(
            "password must be at least 8 characters",
        ));
    }
    if password.len() > MAX_PASSWORD_LEN {
        return Err(ApiError::InvalidInput(
            "password must be at most 128 characters",
        ));
    }
    Ok(())
}

/// Deliberately simple (this is a single-user self-hosted app, not a
/// public signup form) - just closes the gaps `contains('@')` alone lets
/// through (`"a@"`, `"@b"`, `"a b@c.d"`), not a full RFC 5322 parser.
fn is_valid_email(email: &str) -> bool {
    if email.len() > 254 || email.chars().any(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    let Some((_, last_label)) = domain.rsplit_once('.') else {
        return false;
    };
    !last_label.is_empty()
}

fn map_create_user_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(db_error) = &error
        && db_error.is_unique_violation()
    {
        return ApiError::EmailTaken;
    }
    ApiError::Internal(error.into())
}
