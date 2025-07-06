use serde::Serialize;

use super::entity::DictEntity;

/// Dictionary item detail information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictDetailVo {
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

/// Dictionary list response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictListVo {
    pub list: Vec<DictDetailVo>,
    pub total: i64,
}

/// Dictionary option for dropdowns
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictOptionVo {
    pub label: String,
    pub value: String,
}

impl From<DictEntity> for DictDetailVo {
    fn from(entity: DictEntity) -> Self {
        Self {
            id: entity.id,
            dict_type: entity.dict_type,
            label: entity.label,
            value: entity.value,
            is_default: entity.is_default,
        }
    }
}

impl From<DictEntity> for DictOptionVo {
    fn from(entity: DictEntity) -> Self {
        Self { label: entity.label, value: entity.value }
    }
}
