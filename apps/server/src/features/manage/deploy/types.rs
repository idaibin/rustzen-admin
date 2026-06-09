use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum DeployComponent {
    Server,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentItem {
    pub id: i64,
    pub component: DeployComponent,
    pub version: String,
    pub arch: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_hash: String,
    pub is_current: bool,
    pub is_deployed: bool,
    pub is_expired: bool,
    pub deployed_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deployed_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDeploymentsQuery {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub component: Option<DeployComponent>,
    pub is_current: Option<bool>,
    pub is_deployed: Option<bool>,
    pub is_expired: Option<bool>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployVersionRequest {
    pub version_id: Option<i64>,
    pub deployed_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpireVersionRequest {
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeploymentPayload {
    pub component: DeployComponent,
    pub version: String,
    pub arch: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_hash: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DeploymentRow {
    pub id: i64,
    pub component: String,
    pub version: String,
    pub arch: String,
    pub file_path: String,
    pub file_size: i64,
    pub file_hash: String,
    pub is_current: bool,
    pub is_deployed: bool,
    pub is_expired: bool,
    pub deployed_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deployed_by: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
