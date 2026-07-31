use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    Page,
    Api,
    Other,
}

impl EventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Page => "page",
            Self::Api => "api",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub event_name: Option<String>,
    pub event_kind: Option<EventKind>,
    pub path: Option<String>,
    pub visitor_id: Option<String>,
    pub platform: Option<String>,
    pub current: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, FromRow)]
pub struct EventRow {
    pub id: i64,
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
    pub is_error: bool,
    pub properties: String,
    pub occurred_at: String,
    pub received_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: i64,
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
    pub is_error: bool,
    pub properties: Value,
    pub occurred_at: String,
    pub received_at: String,
}

impl TryFrom<EventRow> for Event {
    type Error = serde_json::Error;

    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            event_name: row.event_name,
            visitor_id: row.visitor_id,
            user_id: row.user_id,
            session_id: row.session_id,
            platform: row.platform,
            page_path: row.page_path,
            referrer: row.referrer,
            api_path: row.api_path,
            api_method: row.api_method,
            status_code: row.status_code,
            duration_ms: row.duration_ms,
            is_error: row.is_error,
            properties: serde_json::from_str(&row.properties)?,
            occurred_at: row.occurred_at,
            received_at: row.received_at,
        })
    }
}
