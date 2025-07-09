use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Menu entity (single table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuEntity {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub title: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub permission_code: Option<String>,
}
