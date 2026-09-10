//! Focused runtime configuration for the four Rustzen applications.

#[cfg(any(
    all(feature = "admin", feature = "admin-monitor"),
    all(feature = "admin", feature = "admin-insights"),
    all(feature = "admin-monitor", feature = "admin-insights")
))]
compile_error!("select exactly one Admin configuration profile");

#[cfg(all(
    feature = "admin-insights",
    any(
        feature = "insights",
        feature = "monitor-controller",
        feature = "monitor-agent",
        feature = "reports",
        feature = "notifications"
    )
))]
compile_error!("admin-insights cannot include another process configuration");

#[cfg(any(feature = "admin", feature = "admin-monitor", feature = "admin-insights"))]
mod admin;
#[cfg(feature = "monitor-agent")]
mod agent_activation;
mod contract;
#[cfg(feature = "insights")]
mod insights;
#[cfg(any(feature = "monitor-agent", feature = "monitor-controller"))]
mod monitor;
#[cfg(feature = "reports")]
mod reports;
mod shared;

pub use contract::ConfigContract;
#[cfg(feature = "admin-insights")]
pub use contract::admin_insights_contract;
#[cfg(feature = "admin-monitor")]
pub use contract::admin_monitor_contract;
#[cfg(feature = "monitor-agent")]
pub use contract::monitor_agent_contract;
#[cfg(feature = "monitor-controller")]
pub use contract::monitor_controller_contract;
#[cfg(feature = "notifications")]
pub use contract::notifications_contract;
#[cfg(feature = "reports")]
pub use contract::reports_contract;

#[cfg(any(feature = "admin", feature = "admin-monitor", feature = "admin-insights"))]
pub use admin::AdminConfig;
#[cfg(feature = "monitor-agent")]
pub use agent_activation::{
    AgentEnvironment, ControllerProfile, parse_agent_environment_bytes, read_agent_environment,
    read_controller_profile,
};
#[cfg(feature = "insights")]
pub use insights::InsightsConfig;
#[cfg(feature = "monitor-agent")]
pub use monitor::MonitorAgentConfig;
#[cfg(feature = "monitor-controller")]
pub use monitor::MonitorControllerConfig;
#[cfg(feature = "reports")]
pub use reports::ReportsConfig;
#[cfg(feature = "monitor-agent")]
pub use shared::canonical_monitor_endpoint;
pub use shared::{
    ConfigError, DatabaseConfig, RuntimeConfig, load_dotenv_if_present, valid_monitor_agent_token,
};

/// Fixed retention period for Admin logs, task runs, metrics, events, and reports.
pub const RETENTION_DAYS: u64 = 30;

/// Sets the process time zone before a runtime creates worker threads.
///
/// # Safety
///
/// The caller must invoke this during synchronous process startup, before any
/// other threads can read or write the process environment.
pub unsafe fn initialize_process_timezone(timezone: &str) {
    // SAFETY: upheld by the caller contract above.
    unsafe {
        std::env::set_var("TZ", timezone.trim());
    }
}

#[cfg(all(test, feature = "admin"))]
mod contract_tests {
    use figment::{Figment, providers::Serialized};
    use serde::Serialize;

    use crate::AdminConfig;
    #[cfg(feature = "monitor-controller")]
    use crate::MonitorControllerConfig;

    #[cfg(feature = "monitor-controller")]
    #[derive(Serialize)]
    struct EndpointOverrides<'a> {
        internal_host: &'a str,
        monitor_port: u16,
    }

    #[derive(Serialize)]
    struct LegacyOverrides<'a> {
        app_host: &'a str,
        app_port: u16,
        worker_host: &'a str,
        sqlite_path: &'a str,
    }

    #[cfg(feature = "monitor-controller")]
    #[test]
    fn admin_and_monitor_derive_the_same_fixed_service_endpoint() {
        let overrides = EndpointOverrides { internal_host: "127.0.0.9", monitor_port: 19082 };
        let admin: AdminConfig = Figment::new()
            .merge(Serialized::defaults(&overrides))
            .extract()
            .expect("Admin endpoint overrides");
        let monitor: MonitorControllerConfig = Figment::new()
            .merge(Serialized::defaults(&overrides))
            .extract()
            .expect("Monitor endpoint overrides");

        assert_eq!(admin.monitor_base_url(), "http://127.0.0.9:19082");
        assert_eq!(monitor.bind_address(), "127.0.0.9:19082");
    }

    #[test]
    fn legacy_endpoint_and_database_names_are_not_compatibility_aliases() {
        let config: AdminConfig = Figment::new()
            .merge(Serialized::defaults(LegacyOverrides {
                app_host: "legacy.example",
                app_port: 19001,
                worker_host: "legacy.internal",
                sqlite_path: "/tmp/legacy.db",
            }))
            .extract()
            .expect("ignored legacy values");

        assert_eq!(config.admin_host(), "0.0.0.0");
        assert_eq!(config.admin_port(), 9801);
        assert_eq!(config.internal_host(), "127.0.0.1");
        assert!(config.admin_database_path().ends_with("data/db/admin.db"));
    }

    #[test]
    fn production_env_example_is_the_exact_non_empty_required_set() {
        let values = include_str!("../../../.env.example")
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| line.split_once('=').expect("NAME=value"))
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                ("RUSTZEN_ENV", "production"),
                ("RUSTZEN_RUNTIME_ROOT", "."),
                ("RUSTZEN_JWT_SECRET", "replace-me"),
                ("RUSTZEN_IPC_TOKEN", "replace-me"),
                ("RUSTZEN_MONITOR_AGENT_TOKEN", "replace-me"),
                ("RUSTZEN_MONITOR_NODE_ID", "replace-me"),
                ("RUSTZEN_DEPLOY_SIGNATURE_REQUIRED", "true"),
                ("RUSTZEN_DEPLOY_VERIFY_KEY", "replace-me"),
            ]
        );
        assert!(values.iter().all(|(_, value)| !value.trim().is_empty()));
    }
}
