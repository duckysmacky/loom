use axum::{Json, http::StatusCode, response::IntoResponse, response::Response};
use serde_json::json;

/// Error shape for the auth handlers only - not the app's eventual global
/// error type (Phase 7 "Hardening" owns generalizing error responses).
pub enum AuthError {
    InvalidCredentials,
    EmailTaken,
    InvalidInput(&'static str),
    MissingToken,
    InvalidToken,
    Internal(anyhow::Error),
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid email or password"),
            Self::EmailTaken => (StatusCode::CONFLICT, "email already registered"),
            Self::InvalidInput(message) => (StatusCode::BAD_REQUEST, message),
            Self::MissingToken | Self::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "invalid or missing token")
            }
            Self::Internal(error) => {
                tracing::error!(?error, "auth error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AuthError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.into())
    }
}

/// Error shape for the node/topic handlers only - mirrors `AuthError`
/// deliberately rather than sharing it; Phase 7 "Hardening" owns
/// generalizing error responses across the app.
pub enum ApiError {
    NotFound,
    Conflict(&'static str),
    InvalidInput(&'static str),
    Internal(anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found"),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::InvalidInput(message) => (StatusCode::BAD_REQUEST, message),
            Self::Internal(error) => {
                tracing::error!(?error, "api error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal error")
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        Self::Internal(error.into())
    }
}
