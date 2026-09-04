mod common;
mod features;
mod infra;
mod middleware;
#[cfg(feature = "full")]
mod openapi;

#[cfg(all(feature = "full", feature = "monitor-distribution"))]
compile_error!("full and monitor-distribution are mutually exclusive Admin compositions");
#[cfg(not(any(feature = "full", feature = "monitor-distribution")))]
compile_error!("select exactly one Admin composition feature");

#[cfg(feature = "full")]
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
    let command = Command::parse(std::env::args().skip(1))?;
    #[cfg(feature = "monitor-distribution")]
    if command == Command::ContractSelected {
        println!("{}", crate::infra::app::selected_contract_json()?);
        return Ok(());
    }
    #[cfg(feature = "monitor-distribution")]
    if command == Command::ContractConfigSelected {
        println!("{}", serde_json::to_string(&rustzen_config::admin_monitor_contract())?);
        return Ok(());
    }
    // load env
    rustzen_config::load_dotenv_if_present()?;
    #[cfg(feature = "full")]
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
            #[cfg(feature = "monitor-distribution")]
            Command::ContractSelected => unreachable!("contract mode exits before runtime startup"),
            #[cfg(feature = "monitor-distribution")]
            Command::ContractConfigSelected => {
                unreachable!("contract mode exits before runtime startup")
            }
            #[cfg(feature = "full")]
            Command::UpdateWorker(id) => DeployService::run_update_worker(id).await,
            #[cfg(feature = "full")]
            Command::UpdateRecover => DeployService::recover_interrupted_update_at_boot().await,
            #[cfg(feature = "full")]
            Command::OpenApi => unreachable!("OpenAPI generation exits before runtime startup"),
        }
    })?;

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum Command {
    Serve,
    #[cfg(feature = "monitor-distribution")]
    ContractSelected,
    #[cfg(feature = "monitor-distribution")]
    ContractConfigSelected,
    #[cfg(feature = "full")]
    UpdateWorker(i64),
    #[cfg(feature = "full")]
    UpdateRecover,
    #[cfg(feature = "full")]
    OpenApi,
}

impl Command {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, CommandError> {
        let args = args.into_iter().collect::<Vec<_>>();
        match args.as_slice() {
            [mode] if mode == "serve" => Ok(Self::Serve),
            #[cfg(feature = "monitor-distribution")]
            [domain, mode] if domain == "contract" && mode == "selected" => {
                Ok(Self::ContractSelected)
            }
            #[cfg(feature = "monitor-distribution")]
            [domain, kind, mode]
                if domain == "contract" && kind == "config" && mode == "selected" =>
            {
                Ok(Self::ContractConfigSelected)
            }
            #[cfg(feature = "full")]
            [mode] if mode == "openapi" => Ok(Self::OpenApi),
            #[cfg(feature = "full")]
            [module, mode] if module == "update" && mode == "recover" => Ok(Self::UpdateRecover),
            #[cfg(feature = "full")]
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
        #[cfg(feature = "full")]
        let usage = "usage: rz-admin serve | rz-admin openapi | rz-admin update worker <release-id> | rz-admin update recover";
        #[cfg(not(feature = "full"))]
        let usage = "usage: rz-admin serve | rz-admin contract selected | rz-admin contract config selected";
        formatter.write_str(usage)
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::Command;

    #[test]
    fn parses_admin_commands_only() {
        assert_eq!(Command::parse(["serve".to_string()]).ok(), Some(Command::Serve));
        #[cfg(feature = "full")]
        assert_eq!(Command::parse(["openapi".to_string()]).ok(), Some(Command::OpenApi));
        #[cfg(feature = "full")]
        assert_eq!(
            Command::parse(["update".to_string(), "worker".to_string(), "7".to_string()]).ok(),
            Some(Command::UpdateWorker(7))
        );
        #[cfg(feature = "full")]
        assert_eq!(
            Command::parse(["update".to_string(), "recover".to_string()]).ok(),
            Some(Command::UpdateRecover)
        );
        assert!(Command::parse(["monitor".to_string(), "controller".to_string()]).is_err());
        #[cfg(feature = "monitor-distribution")]
        assert_eq!(
            Command::parse(["contract".to_string(), "selected".to_string()]).ok(),
            Some(Command::ContractSelected)
        );
        #[cfg(feature = "monitor-distribution")]
        assert_eq!(
            Command::parse(["contract".to_string(), "config".to_string(), "selected".to_string()])
                .ok(),
            Some(Command::ContractConfigSelected)
        );
        #[cfg(feature = "monitor-distribution")]
        assert!(Command::parse(["openapi".to_string()]).is_err());
        assert!(Command::parse(std::iter::empty()).is_err());
    }

    #[test]
    fn local_admin_startup_configuration_is_valid() {
        rustzen_config::AdminConfig::local().expect("local Admin startup config");
    }
}
