use chrono::NaiveDateTime;
use serde::Serialize;

use crate::common::api::OptionItem;

use super::entity::RoleEntity;

/// Role detail information
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDetailVo {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub menus: Vec<OptionItem<i64>>,
}

/// Simplified role information (for reference by other modules)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleVo {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}

impl From<RoleEntity> for RoleDetailVo {
    fn from(entity: RoleEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            code: entity.code,
            description: entity.description,
            status: entity.status,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            menus: vec![],
        }
    }
}

impl From<RoleEntity> for RoleVo {
    fn from(entity: RoleEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            code: entity.code,
            description: entity.description,
        }
    }
}
