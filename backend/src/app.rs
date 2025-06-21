use crate::features;
use axum::{Router, response::Json, routing::get};
use serde_json::json;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// 健康检查端点
async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "message": "Rustzen Admin Backend is running",
        "version": "0.1.0"
    }))
}

/// 根路径处理
async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Welcome to Rustzen Admin API",
        "endpoints": {
            "health": "/health",
            "api": "/api"
        }
    }))
}

/// 构建 Axum 应用
pub fn create_app() -> Router {
    // 1. 创建 /sys 路由，并挂载所有业务模块
    let sys_router = Router::new()
        .nest("/user", features::user::router())
        .nest("/role", features::role::router())
        .nest("/menu", features::menu::router())
        .nest("/dict", features::dict::router())
        .nest("/log", features::log::router());

    // 2. 将 /sys 路由挂载到主路由的 /api 下
    let api_router = Router::new().nest("/sys", sys_router);

    // 3. 创建应用根路由，并应用中间件
    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive())
}
