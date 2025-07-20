use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::entity::MenuEntity;

/// Menu detail information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuDetailVo {
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
    pub children: Option<Vec<MenuDetailVo>>,
}

impl From<MenuEntity> for MenuDetailVo {
    fn from(entity: MenuEntity) -> Self {
        Self {
            id: entity.id,
            parent_id: entity.parent_id,
            name: entity.name,
            code: entity.code,
            menu_type: entity.menu_type,
            is_system: entity.is_system,
            sort_order: entity.sort_order,
            status: entity.status,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            children: None,
        }
    }
}
