use std::path::{Path, PathBuf};
use std::time::Duration;

use sqlx::sqlite::SqlitePoolOptions;

pub use sqlx::SqlitePool;

/// Default-sized SQLite connection options for local service startup.
#[derive(Debug, Clone)]
pub struct DatabaseConnectionOptions {
    /// Maximum number of pooled connections.
    pub max_connections: u32,
    /// Minimum number of pooled connections.
    pub min_connections: u32,
    /// Connection acquisition timeout.
    pub connect_timeout: Duration,
    /// Optional idle timeout. `None` disables idle reaping.
    pub idle_timeout: Option<Duration>,
}

impl Default for DatabaseConnectionOptions {
    fn default() -> Self {
        Self {
            max_connections: 4,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Some(Duration::from_secs(600)),
        }
    }
}

/// Builds a SQLite URL from a filesystem path.
pub fn database_url_from_path(path: &Path) -> String {
    if let Some(path) = path.to_str() {
        if path == ":memory:" || path.starts_with("sqlite:") {
            return path.to_string();
        }
    }

    let absolute_path = path.to_path_buf();
    format!("sqlite:///{}", absolute_path.display())
}

/// Create an SQLite pool with explicit options.
pub async fn connect_sqlite_with_options(
    database_url: &str,
    options: DatabaseConnectionOptions,
) -> Result<SqlitePool, sqlx::Error> {
    ensure_database_directory(database_url)?;
    SqlitePoolOptions::new()
        .max_connections(options.max_connections)
        .min_connections(options.min_connections)
        .acquire_timeout(options.connect_timeout)
        .idle_timeout(options.idle_timeout)
        .connect(database_url)
        .await
}

/// Create an SQLite pool with default options.
pub async fn connect_sqlite(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    connect_sqlite_with_options(database_url, DatabaseConnectionOptions::default()).await
}

/// Tests a running connection.
pub async fn test_connection(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

fn ensure_database_directory(database_url: &str) -> Result<(), sqlx::Error> {
    let db_path =
        if let Some(path) = database_url.strip_prefix("sqlite://") { path } else { database_url };

    if db_path == ":memory:" {
        return Ok(());
    }

    let db_path = PathBuf::from(db_path);
    if let Some(parent) = db_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(sqlx::Error::Io)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::database_url_from_path;
    use std::path::Path;

    #[test]
    fn database_url_keeps_explicit_sqlite_memory() {
        assert_eq!(database_url_from_path(Path::new(":memory:")), ":memory:");
    }

    #[test]
    fn database_url_formats_file_path() {
        assert_eq!(database_url_from_path(Path::new("/tmp/data.db")), "sqlite:////tmp/data.db");
    }
}
