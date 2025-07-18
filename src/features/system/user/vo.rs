use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::common::api::OptionItem;

/// Role information
#[derive(Debug, Serialize, Deserialize)]
pub struct RoleVo {
    pub id: i64,
    pub name: String,
    pub code: String,
}

/// User list item
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListVo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub last_login_at: Option<DateTime<Utc>>,
    pub roles: Vec<RoleVo>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User detail information
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDetailVo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub roles: Vec<RoleVo>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User status option
pub type UserStatusOptionVo = OptionItem<i16>;

/// User option
pub type UserOptionVo = OptionItem<i64>;
