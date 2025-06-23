// Menu-related data structures (database models, API request/response bodies) go here.

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Database menu model
#[derive(Debug, Clone, FromRow)]
pub struct MenuEntity {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub title: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub status: i16,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// API response menu model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuResponse {
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
    pub children: Vec<MenuResponse>,
}

/// Create menu request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMenuRequest {
    pub parent_id: Option<i64>,
    pub title: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

/// Update menu request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMenuRequest {
    pub parent_id: Option<i64>,
    pub title: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
}

/// Menu query parameters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuQueryParams {
    pub title: Option<String>,
    pub status: Option<i16>,
}

/// Menu list response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuListResponse {
    pub list: Vec<MenuResponse>,
    pub total: i64,
}

impl From<MenuEntity> for MenuResponse {
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
            children: vec![],
        }
    }
}
