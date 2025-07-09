use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::entity::MenuEntity;

/// Menu detail information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuDetailVo {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub title: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub children: Option<Vec<MenuDetailVo>>,
    pub permission_code: Option<String>,
}

/// Simplified menu information (for reference by other modules)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuVo {
    pub id: i64,
    pub title: String,
    pub path: Option<String>,
}

impl From<MenuEntity> for MenuDetailVo {
    fn from(entity: MenuEntity) -> Self {
        Self {
            id: entity.id,
            parent_id: entity.parent_id,
            title: entity.title,
            path: entity.path,
            component: entity.component,
            icon: entity.icon,
            sort_order: entity.sort_order,
            status: entity.status,
            created_at: DateTime::from_naive_utc_and_offset(entity.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(entity.updated_at, Utc),
            children: None,
            permission_code: entity.permission_code,
        }
    }
}

impl From<MenuEntity> for MenuVo {
    fn from(entity: MenuEntity) -> Self {
        Self { id: entity.id, title: entity.title, path: entity.path }
    }
}
