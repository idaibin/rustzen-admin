use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::entity::RoleEntity;

/// Role detail information
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDetailVo {
    pub id: i64,
    pub role_name: String,
    pub role_code: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub menu_ids: Vec<i64>,
}

/// Simplified role information (for reference by other modules)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleVo {
    pub id: i64,
    pub role_name: String,
    pub role_code: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MenuItemVo {
    pub id: i64,
    pub name: String,
    pub code: String,
}

impl From<RoleEntity> for RoleDetailVo {
    fn from(entity: RoleEntity) -> Self {
        Self {
            id: entity.id,
            role_name: entity.role_name,
            role_code: entity.role_code,
            description: entity.description,
            status: entity.status,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            menu_ids: vec![], // Will be populated in service layer
        }
    }
}

impl From<RoleEntity> for RoleVo {
    fn from(entity: RoleEntity) -> Self {
        Self {
            id: entity.id,
            role_name: entity.role_name,
            role_code: entity.role_code,
            description: entity.description,
        }
    }
}
