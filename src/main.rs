mod common;
mod core;
mod features;
mod middleware;

use crate::core::app::create_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init log
    tracing_subscriber::fmt().with_target(false).compact().init();

    // load env
    dotenvy::dotenv().ok();

    // create server
    create_server().await?;

    Ok(())
}
