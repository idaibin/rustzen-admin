use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListQuery {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub(super) struct IncidentRow {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub details: String,
    pub opened_at: String,
    pub acknowledged_at: Option<String>,
    pub resolved_at: Option<String>,
    pub last_observed_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IncidentSummary {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub kind: String,
    pub title: String,
    pub status: String,
    pub opened_at: String,
    pub acknowledged_at: Option<String>,
    pub resolved_at: Option<String>,
    pub last_observed_at: String,
}

impl From<IncidentRow> for IncidentSummary {
    fn from(row: IncidentRow) -> Self {
        Self {
            id: row.id,
            source_type: row.source_type,
            source_id: row.source_id,
            kind: row.kind,
            title: row.title,
            status: row.status,
            opened_at: row.opened_at,
            acknowledged_at: row.acknowledged_at,
            resolved_at: row.resolved_at,
            last_observed_at: row.last_observed_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IncidentDetail {
    #[serde(flatten)]
    pub summary: IncidentSummary,
    pub details: Value,
    pub node: Option<NodeContext>,
    pub check: Option<CheckContext>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NodeContext {
    pub id: String,
    pub agent_id: String,
    pub hostname: String,
    pub agent_version: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CheckContext {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i64,
    pub last_status: Option<String>,
    pub last_checked_at: Option<String>,
    pub consecutive_failures: i64,
}

#[derive(Debug, Clone, FromRow)]
pub(super) struct ResourceSample {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
}
