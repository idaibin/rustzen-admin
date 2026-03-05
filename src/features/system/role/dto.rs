use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::common::api::OptionItem;

use super::model::RoleWithMenuEntity;

/// Create and update role request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleDto {
    pub name: String,
    pub code: String,
    pub status: i16,
    pub menu_ids: Vec<i64>,
    pub description: Option<String>,
}

/// Update role request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRolePayload {
    pub name: String,
    pub code: String,
    pub status: i16,
    pub menu_ids: Vec<i64>,
    pub description: Option<String>,
}

/// Role list query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleQuery {
    /// The page number to retrieve. Defaults to 1.
    pub current: Option<i64>,
    /// The number of items per page. Defaults to 10.
    pub page_size: Option<i64>,
    /// Filter by role name (case-insensitive search).
    pub role_name: Option<String>,
    /// Filter by role code (case-insensitive search).
    pub role_code: Option<String>,
    /// Filter by role status.
    pub status: Option<String>,
}

/// Role item for list display
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleItemResp {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub menus: Vec<OptionItem<i64>>,
}

impl From<RoleWithMenuEntity> for RoleItemResp {
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
