mod agent;
#[path = "infra/logger.rs"]
mod logger;
pub mod protocol;
mod protocol_contract;

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
    if std::env::args().skip(1).collect::<Vec<_>>() == ["contract", "protocol"] {
        println!("{}", protocol::contract_protocol_output());
        return Ok(());
    }
    rustzen_config::load_dotenv_if_present()?;
    let config = rustzen_config::MonitorAgentConfig::load()?;
    // SAFETY: this runs before Tokio creates worker threads.
    unsafe { rustzen_config::initialize_process_timezone(config.timezone()) };
    let log_dir = config.log_dir();
    let endpoint = protocol::agent_reports_endpoint(&agent_controller_base(
        config.monitor_controller_url.as_deref(),
        config.admin_port(),
    ));
    let agent_token = config.monitor_agent_token.clone();
    let node_id = config.node_id()?.to_string();
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async move {
        let _logging = init_logging(log_dir)?;
        agent::run_agent(node_id, endpoint, agent_token).await
    })
}

fn agent_controller_base(controller_url: Option<&str>, admin_port: u16) -> String {
    controller_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("http://127.0.0.1:{admin_port}"))
}

#[cfg(test)]
mod tests {
    use super::agent_controller_base;

    #[test]
    fn controller_base_uses_configured_admin_port_or_trimmed_override() {
        assert_eq!(agent_controller_base(None, 19081), "http://127.0.0.1:19081");
        assert_eq!(
            agent_controller_base(Some("  https://monitor.example/base/  "), 19081),
            "https://monitor.example/base/"
        );
    }
}
