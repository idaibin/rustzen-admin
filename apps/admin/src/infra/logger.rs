use rustzen_config::RETENTION_DAYS;

use crate::infra::config::CONFIG;

const LOG_FILE_PREFIX: &str = "admin";

pub use rustzen_runtime::FileLoggingGuard as LoggingGuard;

pub fn init_logging() -> Result<LoggingGuard, Box<dyn std::error::Error>> {
    rustzen_runtime::init_file_logging(
        CONFIG.runtime.log_dir().join(LOG_FILE_PREFIX),
        LOG_FILE_PREFIX,
        RETENTION_DAYS,
        "Failed to cleanup expired log files",
    )
}

pub fn init_command_logging() -> Result<(), Box<dyn std::error::Error>> {
    rustzen_runtime::init_stdout_logging()
}
