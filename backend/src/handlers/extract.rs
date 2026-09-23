//! Wrapper extractors that turn axum's own rejections (malformed JSON body,
//! a path segment that doesn't parse as a UUID, a query string with a bad
//! enum value) into `ApiError`'s `{"error": ...}` shape - so every 4xx this
//! API returns, not just the ones our own handlers construct, has the same
//! response shape for the frontend to rely on.

use axum::extract::{FromRequest, FromRequestParts, Path, Query, Request};
use axum::http::request::Parts;
use axum::{Json, RequestPartsExt};
use serde::de::DeserializeOwned;

use super::error::ApiError;

pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        Json::<T>::from_request(req, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(|rejection| ApiError::BadRequest(rejection.to_string()))
    }
}

pub struct ApiPath<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiPath<T>
where
    T: serde::de::DeserializeOwned + Send + 'static,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extract_with_state::<Path<T>, S>(state)
            .await
            .map(|Path(value)| Self(value))
            .map_err(|rejection| ApiError::BadRequest(rejection.to_string()))
    }
}

pub struct ApiQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: serde::de::DeserializeOwned + Send + 'static,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extract_with_state::<Query<T>, S>(state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(|rejection| ApiError::BadRequest(rejection.to_string()))
    }
}
