mod common;
mod features;
mod infra;
mod middleware;
#[cfg(feature = "full")]
mod openapi;

#[cfg(all(feature = "full", feature = "selected-distribution"))]
compile_error!("full and selected-distribution are mutually exclusive Admin compositions");
#[cfg(all(feature = "selected-distribution", feature = "reports-notifications"))]
compile_error!("Reports notifications require the full Admin composition");
#[cfg(all(feature = "analytics-distribution", feature = "notifications"))]
compile_error!("Analytics distribution does not include notifications");
#[cfg(all(feature = "monitor-distribution", feature = "analytics-distribution"))]
compile_error!("select one selected Admin composition feature");
#[cfg(not(any(
    feature = "full",
    feature = "monitor-distribution",
    feature = "analytics-distribution"
)))]
compile_error!("select exactly one reviewed Admin composition feature");

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
    #[cfg(feature = "selected-distribution")]
    if command == Command::ValidateConfig {
        rustzen_config::load_dotenv_if_present()?;
        let _ = CONFIG.admin_database_path();
        return Ok(());
    }
    #[cfg(feature = "selected-distribution")]
    if matches!(command, Command::BindDatabase | Command::ValidateDatabase) {
        rustzen_config::load_dotenv_if_present()?;
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        return runtime.block_on(async {
            if command == Command::BindDatabase {
                let pool = crate::infra::db::create_default_pool().await?;
                crate::infra::db::bind_selected_identity(&pool)
                    .await
                    .map_err(std::io::Error::other)?;
                sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(&pool).await?;
                pool.close().await;
            } else {
                crate::infra::db::verify_selected_database()
                    .await
                    .map_err(std::io::Error::other)?;
            }
            Ok(())
        });
    }
    #[cfg(feature = "selected-distribution")]
    if let Command::ContractSelected(owner) = &command {
        println!(
            "{}",
            crate::infra::app::selected_contract_json(owner).map_err(std::io::Error::other)?
        );
        return Ok(());
    }
    #[cfg(feature = "selected-distribution")]
    if let Command::ContractConfigSelected(owner) = &command {
        let contract = match owner.as_str() {
            "access" => selected_admin_config_contract(),
            #[cfg(feature = "notifications")]
            "notifications" => rustzen_config::notifications_contract(),
            _ => return Err(std::io::Error::other("config owner is not selected").into()),
        };
        println!("{}", serde_json::to_string(&contract)?);
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
            #[cfg(feature = "selected-distribution")]
            Command::BootstrapOwner => {
                crate::infra::bootstrap_owner::replace_seed_owner().await.map_err(|error| {
                    Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>
                })
            }
            #[cfg(feature = "selected-distribution")]
            Command::VerifyOwner => {
                crate::infra::bootstrap_owner::verify_owner().await.map_err(|error| {
                    Box::new(std::io::Error::other(error)) as Box<dyn std::error::Error>
                })
            }
            #[cfg(feature = "selected-distribution")]
            Command::ValidateConfig => unreachable!("validation mode exits before runtime startup"),
            #[cfg(feature = "selected-distribution")]
            Command::ValidateDatabase => {
                unreachable!("database validation exits before runtime startup")
            }
            #[cfg(feature = "selected-distribution")]
            Command::BindDatabase => unreachable!("database binding exits before runtime startup"),
            #[cfg(feature = "selected-distribution")]
            Command::ContractSelected(_) => {
                unreachable!("contract mode exits before runtime startup")
            }
            #[cfg(feature = "selected-distribution")]
            Command::ContractConfigSelected(_) => {
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
    #[cfg(feature = "selected-distribution")]
    BootstrapOwner,
    #[cfg(feature = "selected-distribution")]
    VerifyOwner,
    #[cfg(feature = "selected-distribution")]
    ValidateConfig,
    #[cfg(feature = "selected-distribution")]
    ValidateDatabase,
    #[cfg(feature = "selected-distribution")]
    BindDatabase,
    #[cfg(feature = "selected-distribution")]
    ContractSelected(String),
    #[cfg(feature = "selected-distribution")]
    ContractConfigSelected(String),
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
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "bootstrap-owner" => Ok(Self::BootstrapOwner),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "verify-owner" => Ok(Self::VerifyOwner),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "validate-config" => Ok(Self::ValidateConfig),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "validate-database" => Ok(Self::ValidateDatabase),
            #[cfg(feature = "selected-distribution")]
            [mode] if mode == "bind-database" => Ok(Self::BindDatabase),
            #[cfg(feature = "selected-distribution")]
            [domain, mode, owner]
                if domain == "contract"
                    && mode == "selected"
                    && (owner == "admin" || owner == "notifications") =>
            {
                Ok(Self::ContractSelected(owner.clone()))
            }
            #[cfg(feature = "selected-distribution")]
            [domain, kind, mode, owner]
                if domain == "contract"
                    && kind == "config"
                    && mode == "selected"
                    && selected_config_owner(owner) =>
            {
                Ok(Self::ContractConfigSelected(owner.clone()))
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

#[cfg(feature = "selected-distribution")]
fn selected_config_owner(owner: &str) -> bool {
    owner == "access" || (cfg!(feature = "notifications") && owner == "notifications")
}

#[cfg(feature = "selected-distribution")]
fn selected_admin_config_contract() -> rustzen_config::ConfigContract {
    #[cfg(feature = "monitor-distribution")]
    {
        rustzen_config::admin_monitor_contract()
    }
    #[cfg(feature = "analytics-distribution")]
    {
        rustzen_config::admin_insights_contract()
    }
}

#[derive(Debug)]
struct CommandError;

impl std::fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(feature = "full")]
        let usage = "usage: rz-admin serve | rz-admin openapi | rz-admin update worker <release-id> | rz-admin update recover";
        #[cfg(not(feature = "full"))]
        let usage = "usage: rz-admin serve | rz-admin bootstrap-owner | rz-admin verify-owner | rz-admin validate-config | rz-admin bind-database | rz-admin validate-database | rz-admin contract selected <admin|notifications> | rz-admin contract config selected <access|notifications>";
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
        #[cfg(feature = "selected-distribution")]
        assert_eq!(
            Command::parse(["contract".to_string(), "selected".to_string(), "admin".to_string(),])
                .ok(),
            Some(Command::ContractSelected("admin".into()))
        );
        #[cfg(feature = "selected-distribution")]
        assert_eq!(
            Command::parse(["validate-database".to_string()]).ok(),
            Some(Command::ValidateDatabase)
        );
        #[cfg(feature = "selected-distribution")]
        assert_eq!(Command::parse(["bind-database".to_string()]).ok(), Some(Command::BindDatabase));
        #[cfg(feature = "selected-distribution")]
        assert_eq!(
            Command::parse([
                "contract".to_string(),
                "config".to_string(),
                "selected".to_string(),
                "access".to_string()
            ])
            .ok(),
            Some(Command::ContractConfigSelected("access".into()))
        );
        #[cfg(all(feature = "selected-distribution", feature = "notifications"))]
        assert_eq!(
            Command::parse([
                "contract".to_string(),
                "config".to_string(),
                "selected".to_string(),
                "notifications".to_string()
            ])
            .ok(),
            Some(Command::ContractConfigSelected("notifications".into()))
        );
        #[cfg(all(feature = "selected-distribution", not(feature = "notifications")))]
        assert!(
            Command::parse([
                "contract".to_string(),
                "config".to_string(),
                "selected".to_string(),
                "notifications".to_string()
            ])
            .is_err()
        );
        #[cfg(feature = "selected-distribution")]
        assert!(Command::parse(["openapi".to_string()]).is_err());
        assert!(Command::parse(std::iter::empty()).is_err());
    }

    #[test]
    fn local_admin_startup_configuration_is_valid() {
        rustzen_config::AdminConfig::local().expect("local Admin startup config");
    }
}
