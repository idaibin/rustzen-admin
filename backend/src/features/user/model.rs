// backend/src/features/user/model.rs

// 在这里定义与用户相关的数据库模型、API 请求体和响应体。
// 例如:
//
// #[derive(serde::Serialize, serde::Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct User {
//   pub id: i32,
//   pub user_name: String,
// }

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub user_name: String,
    pub role_ids: Vec<i32>,
}
