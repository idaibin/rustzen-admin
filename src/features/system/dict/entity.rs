use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Dictionary item entity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DictEntity {
    pub id: i64,
    /// The type of the dictionary, used to group related items (e.g., "user_status").
    pub dict_type: String,
    /// The display text for the item (e.g., "Active").
    pub label: String,
    /// The actual value of the item (e.g., "1").
    pub value: String,
    /// The status of the item.
    pub status: i16,
    /// The description of the item.
    pub description: Option<String>,
    /// The sort order of the item.
    pub sort_order: i32,
    /// The last update time.
    pub updated_at: NaiveDateTime,
}
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DictOptionEntity {
    pub label: String,
    pub value: String,
}
