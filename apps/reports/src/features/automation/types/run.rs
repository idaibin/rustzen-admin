use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: String,
    pub flow_id: String,
    pub status: String,
    #[serde(skip_serializing)]
    pub input_json: String,
    pub error: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateRun {
    pub flow_id: String,
    #[serde(default = "empty_object")]
    pub input: Value,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStep {
    pub id: i64,
    pub run_id: String,
    pub step_index: i64,
    pub action: String,
    pub status: String,
    pub duration_ms: Option<i64>,
    pub message: Option<String>,
    pub created_at: String,
}

#[derive(Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Artifact {
    pub id: String,
    pub run_id: String,
    pub kind: String,
    pub file_name: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub run_retention_days: i64,
    pub artifact_retention_days: i64,
    pub default_step_timeout_seconds: i64,
    pub max_run_timeout_seconds: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationSettings {
    pub timezone: String,
}

fn empty_object() -> Value {
    Value::Object(Default::default())
}
