use axum::{Json, http::StatusCode, response::IntoResponse, response::Response};
use serde_json::json;

/// The app's single error type - every handler returns this. Unified in
/// Phase 7 ("Hardening") from what used to be two near-identical types
/// (auth-only vs. node/topic-only); no reason left to keep them separate
/// once every handler needed the same shape.
pub enum ApiError {
    NotFound,
    Conflict(&'static str),
    InvalidInput(&'static str),
    /// Like `InvalidInput`, but for a message built at runtime - notably
    /// axum's own extractor rejections (malformed JSON, a path segment
    /// that doesn't parse, a bad query param), whose messages aren't
    /// `'static`.
    BadRequest(String),
    InvalidCredentials,
    EmailTaken,
    MissingToken,
    InvalidToken,
    Forbidden(&'static str),
    TooManyRequests,
    Internal(anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            Self::Conflict(message) => (StatusCode::CONFLICT, message.to_string()),
            Self::InvalidInput(message) => (StatusCode::BAD_REQUEST, message.to_string()),
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid email or password".to_string(),
            ),
            Self::EmailTaken => (StatusCode::CONFLICT, "email already registered".to_string()),
            Self::MissingToken | Self::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "invalid or missing token".to_string(),
            ),
            Self::Forbidden(message) => (StatusCode::FORBIDDEN, message.to_string()),
            Self::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "too many requests".to_string(),
            ),
            Self::Internal(error) => {
                tracing::error!(?error, "api error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_string(),
                )
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
