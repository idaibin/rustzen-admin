use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Menu entity (single table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuEntity {
    pub id: i64,
    pub parent_id: i64,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub status: i16,
    pub is_system: bool,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
