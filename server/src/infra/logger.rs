use std::{
    fs,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{Local, NaiveDate};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt::writer::MakeWriterExt};

use crate::infra::config::CONFIG;

pub struct LoggingGuard {
    _file_guard: WorkerGuard,
}

pub fn init_logging() -> Result<LoggingGuard, Box<dyn std::error::Error>> {
    fs::create_dir_all(&CONFIG.log_dir)?;

    let file_appender = DailyLogWriter::new(&CONFIG.log_dir, &CONFIG.log_file_prefix)?;
    let (file_writer, file_guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = match std::env::var("RUST_LOG") {
        Ok(value) if !value.trim().is_empty() => EnvFilter::try_new(value)?,
        _ => EnvFilter::try_new("info")?,
    };
    let writer = std::io::stdout.and(file_writer);

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .with_ansi(false)
        .with_writer(writer)
        .init();

    cleanup_expired_logs()?;
    spawn_log_cleanup_task();

    tracing::info!(
        log_dir = %CONFIG.log_dir,
        log_file_prefix = %CONFIG.log_file_prefix,
        log_retention_days = CONFIG.log_retention_days,
        "Logging initialized"
    );

    Ok(LoggingGuard { _file_guard: file_guard })
}

fn spawn_log_cleanup_task() {
    tokio::spawn(async move {
        let interval = Duration::from_secs(24 * 60 * 60);
        loop {
            tokio::time::sleep(interval).await;

            if let Err(error) = cleanup_expired_logs() {
                tracing::error!(%error, "Failed to cleanup expired log files");
            }
        }
    });
}

fn cleanup_expired_logs() -> Result<(), Box<dyn std::error::Error>> {
    let today = Local::now().date_naive();
    let mut scanned = 0_u64;
    let mut deleted = 0_u64;

    for entry in fs::read_dir(&CONFIG.log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(file_date) = parse_log_date(&path, &CONFIG.log_file_prefix) else {
            continue;
        };

        scanned += 1;

        if !is_expired_log(file_date, today, CONFIG.log_retention_days) {
            continue;
        }

        fs::remove_file(&path)?;
        deleted += 1;
    }

    let cutoff = today - chrono::Days::new(CONFIG.log_retention_days);

    tracing::info!(
        log_dir = %CONFIG.log_dir,
        log_file_prefix = %CONFIG.log_file_prefix,
        retention_days = CONFIG.log_retention_days,
        cutoff = %cutoff,
        scanned,
        deleted,
        kept = scanned.saturating_sub(deleted),
        "Log cleanup completed"
    );

    Ok(())
}

fn is_expired_log(file_date: NaiveDate, today: NaiveDate, retention_days: u64) -> bool {
    let cutoff = today - chrono::Days::new(retention_days);
    file_date < cutoff
}

fn parse_log_date(path: &Path, log_file_prefix: &str) -> Option<NaiveDate> {
    let file_name = path.file_name()?.to_str()?;
    let prefix = format!("{log_file_prefix}-");
    let date = file_name.strip_prefix(&prefix)?.strip_suffix(".log")?;
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

struct DailyLogWriter {
    log_dir: PathBuf,
    log_file_prefix: String,
    current_date: NaiveDate,
    file: File,
}

impl DailyLogWriter {
    fn new(log_dir: &str, log_file_prefix: &str) -> Result<Self, io::Error> {
        let current_date = Local::now().date_naive();
        let file = open_log_file(Path::new(log_dir), log_file_prefix, current_date)?;

        Ok(Self {
            log_dir: PathBuf::from(log_dir),
            log_file_prefix: log_file_prefix.to_string(),
            current_date,
            file,
        })
    }

    fn rotate_if_needed(&mut self) -> Result<(), io::Error> {
        let today = Local::now().date_naive();
        if today == self.current_date {
            return Ok(());
        }

        self.file.flush()?;
        self.file = open_log_file(&self.log_dir, &self.log_file_prefix, today)?;
        self.current_date = today;
        Ok(())
    }
}

impl Write for DailyLogWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.rotate_if_needed()?;
        self.file.write(buf)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.file.flush()
    }
}

fn open_log_file(
    log_dir: &Path,
    log_file_prefix: &str,
    date: NaiveDate,
) -> Result<File, io::Error> {
    let file_name = format!("{log_file_prefix}-{}.log", date.format("%Y-%m-%d"));
    OpenOptions::new().create(true).append(true).open(log_dir.join(file_name))
}

#[cfg(test)]
mod tests {
    use super::{is_expired_log, parse_log_date};
    use chrono::{Days, Local, NaiveDate};
    use std::path::PathBuf;

    #[test]
    fn parse_log_date_returns_date_for_matching_file_name() {
        let path = PathBuf::from("logs/server-2026-04-03.log");
        let file_date = parse_log_date(&path, "server").expect("expected log date");

        assert_eq!(file_date, NaiveDate::from_ymd_opt(2026, 4, 3).unwrap());
    }

    #[test]
    fn parse_log_date_ignores_non_matching_file_name() {
        let path = PathBuf::from("logs/other-2026-04-03.log");

        assert!(parse_log_date(&path, "server").is_none());
    }

    #[test]
    fn is_expired_log_matches_retention_window() {
        let today = Local::now().date_naive();
        let expired_date = today.checked_sub_days(Days::new(10)).unwrap();
        let recent_date = today.checked_sub_days(Days::new(1)).unwrap();

        assert!(is_expired_log(expired_date, today, 7));
        assert!(!is_expired_log(recent_date, today, 7));
        assert!(!is_expired_log(today, today, 7));
    }
}
