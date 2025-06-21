use std::net::SocketAddr;
use tracing::info;

pub mod app;
pub mod common;
pub mod core;
pub mod features;

#[tokio::main]
async fn main() {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false) // 禁用 tracing target
        .compact() // 使用紧凑格式
        .init();

    // 构建应用
    let app = app::create_app().await;

    // 定义监听地址和端口
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    info!("🚀 Server listening on http://{}", addr);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
