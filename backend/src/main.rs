use axum::extract::Extension;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod app;
pub mod common;
pub mod core;
pub mod features;

#[tokio::main]
async fn main() {
    // 加载 .env 文件
    dotenvy::dotenv().ok();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 创建数据库连接池
    let pool = core::db::create_default_pool().await.expect("Failed to create database pool");

    // 测试数据库连接
    core::db::test_connection(&pool).await.expect("Failed to connect to database");

    // 获取服务器配置
    let host = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("APP_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse::<u16>()
        .expect("APP_PORT must be a valid port number");

    let addr = format!("{}:{}", host, port);

    // 创建应用
    let app = app::create_app().layer(Extension(pool));

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Failed to bind to address");

    tracing::info!("服务器启动在 http://{}", addr);

    axum::serve(listener, app).await.expect("Failed to start server");
}
