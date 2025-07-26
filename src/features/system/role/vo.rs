use super::entity::RoleWithMenuEntity;
use crate::common::api::OptionItem;

use chrono::NaiveDateTime;
use serde::Serialize;

/// Role item for list display
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleItemVo {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub menus: Vec<OptionItem<i64>>,
}

impl From<RoleWithMenuEntity> for RoleItemVo {
    fn from(role: RoleWithMenuEntity) -> Self {
        Self {
            id: role.id,
            name: role.name,
            code: role.code,
            description: role.description,
            status: role.status,
            created_at: role.created_at,
            updated_at: role.updated_at,
            menus: serde_json::from_value::<Vec<OptionItem<i64>>>(role.menus).unwrap_or_default(),
        }
    }
}
