use axum::{
    Router,
    extract::State,
    http::HeaderMap,
    response::Json,
    routing::{get, post},
};
use sqlx::PgPool;

use crate::common::api::ApiResponse;
use crate::features::auth::model::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserInfoResponse,
};
use crate::features::auth::service::AuthService;

/// 认证路由
pub fn auth_routes() -> Router<PgPool> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/userinfo", get(get_user_info))
}

/// 用户注册
async fn register(
    State(pool): State<PgPool>,
    Json(request): Json<RegisterRequest>,
) -> Json<ApiResponse<RegisterResponse>> {
    match AuthService::register(&pool, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("注册失败: {}", e);
            ApiResponse::fail(500, "注册失败".to_string())
        }
    }
}

/// 用户登录
async fn login(
    State(pool): State<PgPool>,
    Json(request): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    match AuthService::login(&pool, request).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("登录失败: {}", e);
            ApiResponse::fail(500, "登录失败".to_string())
        }
    }
}

/// 用户登出
async fn logout() -> Json<ApiResponse<()>> {
    // 简单的登出响应，实际的 token 失效由前端处理
    ApiResponse::success(())
}

/// 获取用户信息
async fn get_user_info(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Json<ApiResponse<UserInfoResponse>> {
    // 从 Authorization header 中提取 token
    let token = match headers.get("authorization") {
        Some(header_value) => {
            let auth_str = header_value.to_str().unwrap_or("");
            if auth_str.starts_with("Bearer ") {
                &auth_str[7..]
            } else {
                return ApiResponse::fail(1002, "Token 格式错误".to_string());
            }
        }
        None => {
            return ApiResponse::fail(1002, "缺少 Authorization header".to_string());
        }
    };

    // 验证 token
    let claims = match AuthService::verify_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            return ApiResponse::fail(1002, "Token 无效或过期".to_string());
        }
    };

    // 获取用户信息
    let user_id: i64 = match claims.sub.parse() {
        Ok(id) => id,
        Err(_) => {
            return ApiResponse::fail(1002, "Token 格式错误".to_string());
        }
    };

    match AuthService::get_user_info(&pool, user_id).await {
        Ok(response) => response,
        Err(e) => {
            tracing::error!("获取用户信息失败: {}", e);
            ApiResponse::fail(500, "获取用户信息失败".to_string())
        }
    }
}
