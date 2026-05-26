mod common;
mod features;
mod infra;
mod middleware;

use crate::infra::app::run_server;
use crate::infra::logger::init_logging;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load env
    dotenvy::dotenv().ok();

    // init log
    let _logging = init_logging()?;

    // create server
    run_server().await?;

    Ok(())
}
