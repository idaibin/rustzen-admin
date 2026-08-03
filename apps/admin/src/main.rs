mod common;
mod features;
mod infra;
mod middleware;
mod openapi;

use crate::features::manage::deploy::service::DeployService;
use crate::infra::app::run_server;
use crate::infra::config::CONFIG;
use crate::infra::logger::init_logging;

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_RELEASE_MARKER: &str = concat!(
    "RUSTZEN_RELEASE_MARKER\n",
    "artifact=rz-bundle-member\n",
    "binary=rz-admin\n",
    "version=",
    env!("CARGO_PKG_VERSION"),
    "\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load env
    rustzen_config::load_dotenv_if_present()?;
    let command = Command::parse(std::env::args().skip(1))?;
    if command == Command::OpenApi {
        println!("{}", openapi::normalized_json()?);
        return Ok(());
    }
    // SAFETY: this runs in synchronous main before Tokio creates worker threads.
    unsafe { rustzen_config::initialize_process_timezone(CONFIG.timezone()) };
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;

    runtime.block_on(async move {
        // init log
        let _logging = init_logging()?;

        match command {
            Command::Serve => run_server().await,
            Command::UpdateWorker(id) => DeployService::run_update_worker(id).await,
            Command::UpdateRecover => DeployService::recover_interrupted_update_at_boot().await,
            Command::OpenApi => unreachable!("OpenAPI generation exits before runtime startup"),
        }
    })?;

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Command {
    Serve,
    UpdateWorker(i64),
    UpdateRecover,
    OpenApi,
}

impl Command {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, CommandError> {
        let args = args.into_iter().collect::<Vec<_>>();
        match args.as_slice() {
            [mode] if mode == "serve" => Ok(Self::Serve),
            [mode] if mode == "openapi" => Ok(Self::OpenApi),
            [module, mode] if module == "update" && mode == "recover" => Ok(Self::UpdateRecover),
            [module, mode, id] if module == "update" && mode == "worker" => id
                .parse::<i64>()
                .ok()
                .filter(|id| *id > 0)
                .map(Self::UpdateWorker)
                .ok_or(CommandError),
            _ => Err(CommandError),
        }
    }
}

#[derive(Debug)]
struct CommandError;

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(
            "usage: rz-admin serve | rz-admin openapi | rz-admin update worker <release-id> | rz-admin update recover",
        )
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn parses_admin_commands_only() {
        assert_eq!(Command::parse(["serve".to_string()]).ok(), Some(Command::Serve));
        assert_eq!(Command::parse(["openapi".to_string()]).ok(), Some(Command::OpenApi));
        assert_eq!(
            Command::parse(["update".to_string(), "worker".to_string(), "7".to_string()]).ok(),
            Some(Command::UpdateWorker(7))
        );
        assert_eq!(
            Command::parse(["update".to_string(), "recover".to_string()]).ok(),
            Some(Command::UpdateRecover)
        );
        assert!(Command::parse(["monitor".to_string(), "controller".to_string()]).is_err());
        assert!(Command::parse(std::iter::empty()).is_err());
    }

    #[test]
    fn local_admin_startup_configuration_is_valid() {
        rustzen_config::AdminConfig::local().expect("local Admin startup config");
    }
}
