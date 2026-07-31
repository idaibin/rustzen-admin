use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Menu row from the database.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuRow {
    pub id: i64,
    pub parent_id: i64,
    pub parent_code: Option<String>,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub status: i16,
    pub is_system: bool,
    pub is_manual: bool,
    pub sort_order: i32,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub module_id: Option<String>,
    pub module_menu_code: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Update menu request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMenuPayload {
    pub name: String,
    pub sort_order: i16,
    pub status: i16,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuOptionResp {
    pub label: String,
    pub value: i64,
    pub code: String,
    pub is_system: bool,
    pub module_id: Option<String>,
    pub module_menu_code: Option<String>,
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
    pub is_manual: bool,
    pub sort_order: i32,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub module_id: Option<String>,
    pub module_menu_code: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub children: Option<Vec<MenuItemResp>>,
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

/// Menu repository list query.
#[derive(Debug, Clone)]
pub struct MenuListQuery {
    pub name: Option<String>,
    pub code: Option<String>,
    pub status: Option<i16>,
}

impl From<MenuRow> for MenuItemResp {
    fn from(entity: MenuRow) -> Self {
        Self {
            id: entity.id,
            parent_id: entity.parent_id,
            name: entity.name,
            code: entity.code,
            menu_type: entity.menu_type,
            is_system: entity.is_system,
            is_manual: entity.is_manual,
            sort_order: entity.sort_order,
            path: entity.path,
            icon: entity.icon,
            module_id: entity.module_id,
            module_menu_code: entity.module_menu_code,
            is_active: entity.is_active,
            status: entity.status,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            children: None,
        }
    }
}
