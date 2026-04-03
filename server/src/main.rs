mod common;
mod features;
mod infra;
mod middleware;

use crate::infra::app::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init log
    tracing_subscriber::fmt().with_target(false).compact().init();

    // load env
    dotenvy::dotenv().ok();

    // create server
    run_server().await?;

    Ok(())
}
