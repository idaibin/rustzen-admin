use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    response::Json,
    routing::get,
};
use serde_json::json;
use tower_http::cors::CorsLayer;

use crate::core::db::{create_default_pool, test_connection};
use crate::features::auth::routes::auth_routes;
use crate::features::system::system_routes;

/// 创建并启动服务器
pub async fn create_server() -> Result<(), Box<dyn std::error::Error>> {
    // 创建数据库连接池
    let pool = create_default_pool().await?;

    // 测试数据库连接
    test_connection(&pool).await?;

    // CORS 配置
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT]);

    // 创建应用路由 - 所有API注册都在这里
    let app = Router::new()
        .route("/", get(root))
        .nest("/api", Router::new().nest("/auth", auth_routes()).nest("/system", system_routes()))
        // 添加 CORS 中间件
        .layer(cors)
        .with_state(pool);

    // 启动服务器
    let addr = get_addr().await;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 服务器启动成功，监听地址: http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_addr() -> String {
    let host = std::env::var("APP_HOST").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("APP_PORT").unwrap_or("8000".to_string());
    format!("{}:{}", host, port)
}

/// 根路径处理
async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Welcome to rustzen-admin API",
        "version": "0.1.0",
        "description": "基于 Rust + Axum + SQLx + PostgreSQL 的后台管理系统"
    }))
}
