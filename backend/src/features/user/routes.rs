// src/features/user/routes.rs

use crate::common::response::ApiResponse;
use crate::features::user::model::User;
use axum::{Json, Router, extract::Extension, routing::get};
use sqlx::PgPool;

/// 用户模块路由
pub fn router() -> Router {
    Router::new().route("/", get(get_user_list))
}

// 占位符: 获取用户列表 - 演示数据库连接池使用
async fn get_user_list(Extension(pool): Extension<PgPool>) -> Json<ApiResponse<Vec<User>>> {
    // 测试数据库连接
    match sqlx::query("SELECT 1 as test_column").fetch_one(&pool).await {
        Ok(_) => {
            tracing::info!("数据库连接测试成功");
            // 返回模拟数据
            let mock_users = vec![
                User { id: 1, user_name: "Admin".to_string(), role_ids: vec![1] },
                User { id: 2, user_name: "Editor".to_string(), role_ids: vec![2] },
                User { id: 3, user_name: "Guest".to_string(), role_ids: vec![3] },
            ];
            ApiResponse::success(mock_users)
        }
        Err(e) => {
            tracing::error!("数据库连接失败: {:?}", e);
            ApiResponse::fail(500, "数据库连接失败".to_string())
        }
    }
}
