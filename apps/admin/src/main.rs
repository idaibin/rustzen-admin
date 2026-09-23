mod command;
mod common;
mod features;
mod infra;
mod middleware;
mod openapi;

use crate::features::manage::deploy::service::DeployService;
use crate::infra::app::run_server;
use crate::infra::config::CONFIG;
use crate::infra::logger::{init_command_logging, init_logging};
use command::Command;

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_RELEASE_MARKER: &str = concat!(
    "RUSTZEN_RELEASE_MARKER\n",
    "artifact=rz-bundle-member\n",
    "binary=rz-admin\n",
    "version=",
    env!("CARGO_PKG_VERSION"),
    "\nfrontend_sha256=",
    env!("RUSTZEN_RELEASE_FRONTEND_DIGEST"),
    "\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::parse(std::env::args().skip(1))?;
    rustzen_config::load_dotenv_if_present()?;
    if command == Command::OpenApi {
        println!("{}", openapi::normalized_json()?);
        return Ok(());
    }
    unsafe { rustzen_config::initialize_process_timezone(CONFIG.timezone()) };
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        match command {
            Command::Serve => {
                let _logging = init_logging()?;
                run_server().await
            }
            Command::UpdateWorker(id) => {
                init_command_logging()?;
                DeployService::run_update_worker(id).await
            }
            Command::UpdateRequestWorker => {
                init_command_logging()?;
                DeployService::run_update_request_worker().await
            }
            Command::UpdateRecover => {
                init_command_logging()?;
                DeployService::recover_interrupted_update_at_boot().await
            }
            Command::OpenApi => unreachable!("OpenAPI generation exits before runtime startup"),
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn local_admin_startup_configuration_is_valid() {
        rustzen_config::AdminConfig::local().expect("local Admin startup config");
    }
}
