use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
pub struct InboxListQuery {
    pub cursor: Option<String>,
    pub limit: Option<u16>,
    pub unread_only: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadAllRequest {
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NotificationItem {
    pub id: String,
    pub producer: String,
    pub topic: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub subject_revision: i64,
    pub occurred_at: NaiveDateTime,
    pub accepted_at: NaiveDateTime,
    pub title: String,
    pub summary: String,
    pub read_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct NotificationRow {
    pub id: String,
    pub inbox_seq: i64,
    pub producer: String,
    pub topic: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub subject_revision: i64,
    pub occurred_at: NaiveDateTime,
    pub accepted_at: NaiveDateTime,
    pub title: String,
    pub summary: String,
    pub read_at: Option<NaiveDateTime>,
}

impl From<NotificationRow> for NotificationItem {
    fn from(row: NotificationRow) -> Self {
        Self {
            id: row.id,
            producer: row.producer,
            topic: row.topic,
            subject_kind: row.subject_kind,
            subject_id: row.subject_id,
            subject_revision: row.subject_revision,
            occurred_at: row.occurred_at,
            accepted_at: row.accepted_at,
            title: row.title,
            summary: row.summary,
            read_at: row.read_at,
        }
    }
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InboxListResponse {
    pub items: Vec<NotificationItem>,
    pub next_cursor: Option<String>,
    pub snapshot: String,
    pub revision: i64,
    pub retention_days: u16,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnreadCountResponse {
    pub count: i64,
    pub revision: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadResponse {
    pub id: String,
    pub read_at: NaiveDateTime,
    pub revision: i64,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadAllResponse {
    pub changed: u64,
    pub revision: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(super) struct ReadStateRow {
    pub id: String,
    pub read_at: Option<NaiveDateTime>,
}
