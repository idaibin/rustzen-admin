use chrono::{DateTime, Utc};
use serde::Serialize;

use super::entity::RoleEntity;

/// Role detail information
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDetailVo {
    pub id: i64,
    pub role_name: String,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub menu_ids: Vec<i64>,
}

/// Simplified role information (for reference by other modules)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleVo {
    pub id: i64,
    pub role_name: String,
}

impl From<RoleEntity> for RoleDetailVo {
    fn from(entity: RoleEntity) -> Self {
        Self {
            id: entity.id,
            role_name: entity.role_name,
            status: entity.status,
            created_at: DateTime::from_naive_utc_and_offset(entity.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(entity.updated_at, Utc),
            menu_ids: vec![], // Will be populated in service layer
        }
    }
}

impl From<RoleEntity> for RoleVo {
    fn from(entity: RoleEntity) -> Self {
        Self { id: entity.id, role_name: entity.role_name }
    }
}
