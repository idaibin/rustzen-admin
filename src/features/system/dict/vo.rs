use super::entity::DictEntity;

use chrono::NaiveDateTime;
use serde::Serialize;

/// Dictionary item for list display
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItemVo {
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
    pub description: String,
    /// The sort order of the item.
    pub sort_order: i32,
    /// The last update time.
    pub updated_at: NaiveDateTime,
}

impl From<DictEntity> for DictItemVo {
    fn from(entity: DictEntity) -> Self {
        Self {
            id: entity.id,
            dict_type: entity.dict_type,
            label: entity.label,
            value: entity.value,
            status: entity.status,
            description: entity.description.unwrap_or("".to_string()),
            sort_order: entity.sort_order,
            updated_at: entity.updated_at,
        }
    }
}
