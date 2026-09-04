mod agent;
#[path = "infra/logger.rs"]
mod logger;
pub mod protocol;

use crate::logger::init_logging;

#[used]
#[unsafe(no_mangle)]
pub static RUSTZEN_RELEASE_MARKER: &str = concat!(
    "RUSTZEN_RELEASE_MARKER\n",
    "artifact=rz-bundle-member\n",
    "binary=rz-monitor-agent\n",
    "version=",
    env!("CARGO_PKG_VERSION"),
    "\n",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustzen_config::load_dotenv_if_present()?;
    let config = rustzen_config::MonitorAgentConfig::load()?;
    // SAFETY: this runs before Tokio creates worker threads.
    unsafe { rustzen_config::initialize_process_timezone(config.timezone()) };
    let log_dir = config.log_dir();
    let endpoint = config.agent_reports_endpoint();
    let agent_token = config.monitor_agent_token.clone();
    let node_id = config.node_id()?.to_string();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        let _logging = init_logging(log_dir)?;
        agent::run_agent(node_id, endpoint, agent_token).await
    })
}
