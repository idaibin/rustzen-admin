use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub event_retention_days: i64,
    pub default_query_days: i64,
    pub max_query_days: i64,
    pub max_batch_events: i64,
    pub business_timezone: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionPolicy {
    pub collection_enabled: bool,
    pub project_configured: bool,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectionPolicyUpdate {
    pub collection_enabled: bool,
    pub project_key: Option<String>,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CollectionPolicyRow {
    pub collection_enabled: i64,
    pub project_key_hash: String,
    pub allowed_origins: String,
}
