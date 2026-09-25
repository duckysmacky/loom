use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use super::deserialize_some;

#[derive(Debug, Clone, Serialize, sqlx::FromRow, TS)]
#[ts(export)]
pub struct TopicResponse {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct CreateTopicRequest {
    pub name: String,
    #[serde(default)]
    #[ts(optional = nullable)]
    pub color: Option<String>,
}

#[derive(Debug, Default, Deserialize, TS, JsonSchema)]
#[ts(export)]
pub struct UpdateTopicRequest {
    #[serde(default)]
    #[ts(optional = nullable)]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    #[ts(optional = nullable)]
    pub color: Option<Option<String>>,
}
