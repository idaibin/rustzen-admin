use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::model::MenuEntity;

/// Create menu request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMenuDto {
    pub parent_id: i64,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub sort_order: i16,
    pub status: i16,
}

/// Update menu request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMenuPayload {
    pub parent_id: i64,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub sort_order: i16,
    pub status: i16,
}

/// Menu query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuQuery {
    /// The name of the menu.
    pub name: Option<String>,
    /// The code of the menu.
    pub code: Option<String>,
    /// The status of the menu.
    pub status: Option<String>,
}

/// Menu item for tree list display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItemResp {
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
    pub children: Option<Vec<MenuItemResp>>,
}

impl From<MenuEntity> for MenuItemResp {
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
