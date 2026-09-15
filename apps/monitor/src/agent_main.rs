mod agent;
mod controller_profile;
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
    if std::env::args().skip(1).collect::<Vec<_>>() == ["contract", "config", "selected"] {
        // Canonicalize through serde_json::Value so selected-config stdout matches
        // the sorted-key contract artifact bytes byte for byte.
        println!("{}", serde_json::to_string(&serde_json::to_value(rustzen_config::monitor_agent_contract())?)?);
        return Ok(());
    }
    rustzen_config::load_dotenv_if_present()?;
    let config = paired_config()?;
    if std::env::args().skip(1).collect::<Vec<_>>() == ["contract", "pairing"] {
        return Ok(());
    }
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

fn paired_config() -> Result<rustzen_config::MonitorAgentConfig, Box<dyn std::error::Error>> {
    let config = rustzen_config::MonitorAgentConfig::load()?;
    if !config.runtime.requires_production_secrets() {
        return Ok(config);
    }
    let environment = controller_profile::validate_agent_env(std::path::Path::new("/opt/rz"))?;
    if config.runtime.environment != environment.environment
        || config.node_id()? != environment.node_id
        || config.monitor_agent_token != environment.token
        || config
            .monitor_controller_url
            .as_deref()
            .and_then(|value| rustzen_config::canonical_monitor_endpoint(value).ok())
            .as_deref()
            != Some(environment.endpoint.as_str())
    {
        return Err("Agent runtime configuration differs from activated environment".into());
    }
    controller_profile::validate_profile(
        &config.agent_root().join("controller-profile.json"),
        &agent_controller_base(config.monitor_controller_url.as_deref(), config.admin_port()),
        &config.agent_root(),
    )?;
    controller_profile::validate_running_binary(&config.agent_root())?;
    Ok(config)
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
