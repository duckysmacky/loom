use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Debug, Deserialize, TS)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, TS)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, TS)]
pub struct AuthUserView {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, Serialize, TS)]
pub struct AuthResponse {
    pub access_token: String,
    pub user: AuthUserView,
}

#[derive(Debug, Serialize, TS)]
pub struct RefreshResponse {
    pub access_token: String,
}
