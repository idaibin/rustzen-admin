use super::entity::UserWithRolesEntity;
use crate::common::api::OptionItem;

use chrono::NaiveDateTime;
use serde::Serialize;

/// User item for list display
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserItemVo {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub real_name: Option<String>,
    pub avatar_url: Option<String>,
    pub status: i16,
    pub last_login_at: Option<NaiveDateTime>,
    pub roles: Vec<UserOptionVo>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// User option
pub type UserOptionVo = OptionItem<i64>;

impl From<UserWithRolesEntity> for UserItemVo {
    fn from(user: UserWithRolesEntity) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            real_name: user.real_name,
            avatar_url: user.avatar_url,
            status: user.status,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
            roles: serde_json::from_value::<Vec<UserOptionVo>>(user.roles).unwrap_or_default(),
        }
    }
}
