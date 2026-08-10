use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackInput {
    pub event_name: Option<String>,
    pub visitor_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub platform: Option<String>,
    pub page_path: Option<String>,
    pub referrer: Option<String>,
    pub api_path: Option<String>,
    pub api_method: Option<String>,
    pub status_code: Option<i64>,
    pub duration_ms: Option<u64>,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default = "empty_properties")]
    pub properties: Value,
    pub occurred_at: Option<DateTime<Utc>>,
}

fn empty_properties() -> Value {
    Value::Object(Default::default())
}

pub struct NewEvent {
    pub project_id: String,
    pub event_name: String,
    pub visitor_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub platform: Option<String>,
    pub page_path: Option<String>,
    pub referrer: Option<String>,
    pub api_path: Option<String>,
    pub api_method: Option<String>,
    pub status_code: Option<i64>,
    pub duration_ms: Option<i64>,
    pub is_error: i64,
    pub properties: String,
    pub occurred_at: String,
    pub received_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackAccepted {
    pub accepted: usize,
}
