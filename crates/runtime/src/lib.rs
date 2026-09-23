//! Runtime layout helpers for sqlite-first runtime startup.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{Days, NaiveDate, Utc};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt::writer::MakeWriterExt};

/// Default runtime root used by local development and packaging.
pub const DEFAULT_RUNTIME_ROOT: &str = ".rustzen-admin";

/// Default public files prefix for uploaded assets.
pub const DEFAULT_FILES_PREFIX: &str = "/resources";

/// Keeps the non-blocking file writer alive for a process lifetime.
pub struct FileLoggingGuard {
    _file_guard: WorkerGuard,
}

/// Initializes stdout-only logging for short-lived maintenance commands.
pub fn init_stdout_logging() -> Result<(), Box<dyn std::error::Error>> {
    let filter = EnvFilter::try_new(log_env_filter())?;
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_target(false)
        .compact()
        .try_init()
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    Ok(())
}

/// Initializes the shared stdout and daily-file logging policy for a server process.
pub fn init_file_logging(
    log_dir: impl AsRef<Path>,
    file_prefix: &str,
    retention_days: u64,
    cleanup_error_message: &'static str,
) -> Result<FileLoggingGuard, Box<dyn std::error::Error>> {
    let log_dir = log_dir.as_ref().to_path_buf();
    fs::create_dir_all(&log_dir)?;
    let appender = tracing_appender::rolling::daily(&log_dir, file_prefix);
    let (file_writer, file_guard) = tracing_appender::non_blocking(appender);
    let filter = EnvFilter::try_new(log_env_filter())?;
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_target(false)
        .compact()
        .with_writer(std::io::stdout.and(file_writer))
        .try_init()
        .map_err(|error| std::io::Error::other(error.to_string()))?;

    cleanup_expired_log_files(&log_dir, file_prefix, retention_days, Utc::now().date_naive())?;
    spawn_log_cleanup_task(log_dir, file_prefix.to_string(), retention_days, cleanup_error_message);
    Ok(FileLoggingGuard { _file_guard: file_guard })
}

fn log_env_filter() -> String {
    std::env::var("RUST_LOG")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "info".to_string())
}

fn spawn_log_cleanup_task(
    log_dir: PathBuf,
    file_prefix: String,
    retention_days: u64,
    cleanup_error_message: &'static str,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(24 * 60 * 60)).await;
            if let Err(error) = cleanup_expired_log_files(
                &log_dir,
                &file_prefix,
                retention_days,
                Utc::now().date_naive(),
            ) {
                tracing::error!(%error, "{}", cleanup_error_message);
            }
        }
    });
}

/// Deletes daily log files that fall outside the inclusive retention window.
pub fn cleanup_expired_log_files(
    log_dir: &Path,
    file_prefix: &str,
    retention_days: u64,
    today: NaiveDate,
) -> Result<(), std::io::Error> {
    let cutoff = today - Days::new(retention_days);
    for entry in fs::read_dir(log_dir)? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(date) = name
            .strip_prefix(&format!("{file_prefix}."))
            .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
        else {
            continue;
        };
        if date < cutoff {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Shared runtime paths under a single root path.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RuntimeLayout {
    runtime_root: String,
    files_prefix: String,
}

impl RuntimeLayout {
    /// Creates a layout bound to a runtime root and public files prefix.
    pub fn new(runtime_root: impl Into<String>, files_prefix: impl Into<String>) -> Self {
        let runtime_root = runtime_root.into();
        Self { runtime_root, files_prefix: normalize_prefix(files_prefix.into()) }
    }

    /// Returns the configured runtime root as string.
    pub fn runtime_root(&self) -> &str {
        &self.runtime_root
    }

    /// Root directory path as configured (relative or absolute).
    pub fn runtime_root_dir(&self) -> PathBuf {
        PathBuf::from(&self.runtime_root)
    }

    /// Directory for packaged frontend assets.
    pub fn web_dist_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("web/dist")
    }

    /// Directory for uploaded and runtime data.
    pub fn data_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("data")
    }

    /// Directory for SQLite database files.
    pub fn db_dir(&self) -> PathBuf {
        self.data_dir().join("db")
    }

    /// Directory for logs.
    pub fn log_dir(&self) -> PathBuf {
        self.runtime_root_dir().join("logs")
    }

    /// Avatar file directory.
    pub fn avatars_dir(&self) -> PathBuf {
        self.data_dir().join("avatars")
    }

    /// Upload root directory.
    pub fn uploads_dir(&self) -> PathBuf {
        self.data_dir().join("uploads")
    }

    /// Public avatar prefix for static file route.
    pub fn avatars_prefix(&self) -> String {
        format!("{}/avatars", self.files_prefix.trim_end_matches('/'))
    }

    /// Resolves a runtime path value relative to runtime root.
    pub fn resolve_runtime_path(&self, value: &str) -> PathBuf {
        let value = PathBuf::from(value);
        if value.is_absolute()
            || value.to_str().is_some_and(|raw| raw == ":memory:" || raw.starts_with("sqlite:"))
        {
            value
        } else {
            let root = self.runtime_root_dir();
            if root.is_absolute() {
                root.join(value)
            } else {
                match std::env::current_dir() {
                    Ok(cwd) => cwd.join(root).join(value),
                    Err(_) => root.join(value),
                }
            }
        }
    }
}

/// Resolves a path relative to a runtime root.
///
/// Relative paths use the local working directory when runtime root is relative.
pub fn resolve_path_with_runtime_root(runtime_root: &str, value: &str) -> PathBuf {
    RuntimeLayout::new(runtime_root, DEFAULT_FILES_PREFIX).resolve_runtime_path(value)
}

fn normalize_prefix(value: String) -> String {
    let value = value.trim();
    if value.is_empty() || value == "/" {
        return String::new();
    }
    format!("/{}", value.trim_matches('/'))
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_RUNTIME_ROOT, RuntimeLayout, cleanup_expired_log_files,
        resolve_path_with_runtime_root,
    };
    use chrono::NaiveDate;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn runtime_layout_derives_standard_directories() {
        let layout = RuntimeLayout::new(".rustzen-admin", "/resources");

        assert_eq!(layout.runtime_root_dir(), PathBuf::from(".rustzen-admin"));
        assert_eq!(layout.db_dir(), PathBuf::from(".rustzen-admin/data/db"));
        assert_eq!(layout.web_dist_dir(), PathBuf::from(".rustzen-admin/web/dist"));
        assert_eq!(layout.data_dir(), PathBuf::from(".rustzen-admin/data"));
        assert_eq!(layout.log_dir(), PathBuf::from(".rustzen-admin/logs"));
        assert_eq!(layout.uploads_dir(), PathBuf::from(".rustzen-admin/data/uploads"));
        assert_eq!(layout.avatars_dir(), PathBuf::from(".rustzen-admin/data/avatars"));
        assert_eq!(layout.avatars_prefix(), "/resources/avatars");
    }

    #[test]
    fn runtime_root_default_matches_constant() {
        assert_eq!(DEFAULT_RUNTIME_ROOT, ".rustzen-admin");
    }

    #[test]
    fn resolve_path_prefers_absolute_candidate() {
        let resolved = resolve_path_with_runtime_root(".rustzen-admin", "/tmp/data.db");

        assert_eq!(resolved, PathBuf::from("/tmp/data.db"));
        assert!(Path::new("/tmp/data.db").is_absolute());
    }

    #[test]
    fn cleanup_keeps_log_files_inside_the_retention_window() {
        let dir = std::env::temp_dir().join(format!("rz-logs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("log dir");
        for name in ["monitor.2026-06-13", "monitor.2026-06-14", "unrelated.txt"] {
            std::fs::write(dir.join(name), name).expect("log file");
        }

        cleanup_expired_log_files(
            &dir,
            "monitor",
            30,
            NaiveDate::from_ymd_opt(2026, 7, 14).expect("date"),
        )
        .expect("cleanup");

        assert!(!dir.join("monitor.2026-06-13").exists());
        assert!(dir.join("monitor.2026-06-14").exists());
        assert!(dir.join("unrelated.txt").exists());
        std::fs::remove_dir_all(dir).expect("remove log dir");
    }
}
