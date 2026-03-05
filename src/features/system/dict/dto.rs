use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::model::DictEntity;

/// Create dictionary item request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDictDto {
    /// The type of the dictionary, used to group related items (e.g., "user_status").
    pub dict_type: String,
    /// The display text for the item (e.g., "Active").
    pub label: String,
    /// The actual value of the item (e.g., "1").
    pub value: String,
    /// The status of the item.
    pub status: Option<i16>,
    /// The description of the item.
    pub description: Option<String>,
    /// The sort order of the item.
    pub sort_order: Option<i32>,
}

/// Update dictionary item request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDictPayload {
    pub dict_type: String,
    pub label: String,
    pub value: String,
    pub status: Option<i16>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}
/// Dictionary query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictQuery {
    /// The page number to retrieve. Defaults to 1.
    pub current: Option<i64>,
    /// The number of items per page. Defaults to 10.
    pub page_size: Option<i64>,
    /// Filter by dictionary type.
    pub dict_type: Option<String>,
    /// Filter by label.
    pub label: Option<String>,
    /// Filter by value.
    pub value: Option<String>,
    /// Filter by status.
    pub status: Option<String>,
}

/// Updates the status of a dictionary item.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateDictStatusPayload {
    pub status: i16,
}

/// Dictionary item for list display
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItemResp {
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

impl From<DictEntity> for DictItemResp {
    fn from(entity: DictEntity) -> Self {
        Self {
            id: entity.id,
            dict_type: entity.dict_type,
            label: entity.label,
            value: entity.value,
            status: entity.status,
            description: entity.description.unwrap_or_default(),
            sort_order: entity.sort_order,
            updated_at: entity.updated_at,
        }
    }
}
