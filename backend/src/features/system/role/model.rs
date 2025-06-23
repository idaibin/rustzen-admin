// Role-related data structures (database models, API request/response bodies) go here.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database role model
#[derive(Debug, Clone, FromRow)]
pub struct RoleEntity {
    pub id: i64,
    pub role_name: String,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// API response role model
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleResponse {
    pub id: i64,
    pub role_name: String,
    pub status: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub menu_ids: Vec<i64>,
}

/// Create role request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub role_name: String,
    pub status: Option<i16>,
    pub menu_ids: Vec<i64>,
}

/// Update role request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    pub role_name: Option<String>,
    pub status: Option<i16>,
    pub menu_ids: Option<Vec<i64>>,
}

/// Role query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleQueryParams {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
    pub role_name: Option<String>,
    pub status: Option<i16>,
}

/// Role list response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleListResponse {
    pub list: Vec<RoleResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Simplified role information (for reference by other modules)
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: i64,
    pub role_name: String,
}

impl From<RoleEntity> for RoleResponse {
    fn from(entity: RoleEntity) -> Self {
        Self {
            id: entity.id,
            role_name: entity.role_name,
            status: entity.status,
            created_at: DateTime::from_naive_utc_and_offset(entity.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(entity.updated_at, Utc),
            menu_ids: vec![], // 将在 service 层填充
        }
    }
}

impl From<RoleEntity> for Role {
    fn from(entity: RoleEntity) -> Self {
        Self { id: entity.id, role_name: entity.role_name }
    }
}
