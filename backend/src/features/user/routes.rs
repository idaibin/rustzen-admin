// src/features/user/routes.rs

use crate::common::response::ApiResponse;
use crate::features::user::model::User;
use axum::{Json, Router, routing::get};

/// 用户模块路由
pub fn router() -> Router {
    Router::new().route("/", get(get_user_list))
}

// 占位符: 获取用户列表
async fn get_user_list() -> Json<ApiResponse<Vec<User>>> {
    let mock_users = vec![
        User { id: 1, user_name: "Admin".to_string(), role_ids: vec![1] },
        User { id: 2, user_name: "Editor".to_string(), role_ids: vec![2] },
        User { id: 3, user_name: "Guest".to_string(), role_ids: vec![3] },
    ];
    ApiResponse::success(mock_users)
}
