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
    /// Whether this is the default item of its type.
    pub is_default: bool,
}
