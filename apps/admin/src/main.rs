mod command;
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
use command::Command;

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
    if command == Command::ContractProtocol {
        println!("{}", rustzen_ipc::delegation_protocol_output());
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
        // Canonicalize through serde_json::Value so selected-config stdout matches
        // the sorted-key contract artifact bytes byte for byte.
        println!("{}", serde_json::to_string(&serde_json::to_value(&contract)?)?);
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
            Command::ContractProtocol => unreachable!("contract mode exits before runtime startup"),
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
