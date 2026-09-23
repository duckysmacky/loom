use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::deserialize_some;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, TS)]
pub struct TopicResponse {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS)]
pub struct CreateTopicRequest {
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Default, Deserialize, TS)]
pub struct UpdateTopicRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub color: Option<Option<String>>,
}
