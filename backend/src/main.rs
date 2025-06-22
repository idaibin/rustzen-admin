mod common;
mod core;
mod features;

use crate::core::app::create_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt().with_target(false).compact().init();

    // 加载环境变量
    dotenvy::dotenv().ok();

    // 创建服务器
    create_server().await?;

    Ok(())
}
