use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

// No `Debug` on these two - they carry a plaintext password, and an
// accidental `{:?}` in a log line would leak it.
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, TS)]
#[ts(export)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AuthUserView {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: AuthUserView,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct RefreshResponse {
    pub access_token: String,
}
