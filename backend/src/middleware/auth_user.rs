use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use uuid::Uuid;

use crate::auth::jwt;
use crate::handlers::error::AuthError;
use crate::state::AppState;

/// The `user_id`-extraction middleware every protected route builds on.
/// Every later handler that needs the caller's identity takes this as an
/// argument - `user_id` is then the first argument to whatever repo
/// function it calls, per root CLAUDE.md's access-control model.
pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AuthError::MissingToken)?;

        let claims = jwt::verify_access_token(bearer.token(), &state.jwt_decoding_key)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(AuthUser {
            user_id: claims.sub,
        })
    }
}
