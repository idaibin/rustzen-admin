use rustzen_config::MonitorControllerConfig;
use std::sync::LazyLock;

static CONTROLLER_CONFIG: LazyLock<MonitorControllerConfig> = LazyLock::new(|| {
    MonitorControllerConfig::load().expect("Failed to load Monitor Controller configuration")
});

pub fn controller() -> &'static MonitorControllerConfig {
    &CONTROLLER_CONFIG
}
