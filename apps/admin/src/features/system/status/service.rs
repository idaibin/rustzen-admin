use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::infra::config::CONFIG;
use chrono::Utc;
use tokio::task;
use tracing::warn;

use crate::{
    common::error::ServiceError,
    features::modules::{registry::ModuleRegistry, types::ModuleSpec},
    infra::system_info::{SystemInfo, SystemUtils},
};

use super::types::{
    CpuResourceStatus, DirectoryStorageItem, DiskResourceStatus, LocalResourceStatus,
    MemoryResourceStatus, ModuleDatabaseStatus, SqliteStorageStatus, SystemStatusOverview,
    SystemStorageStatus,
};

pub struct SystemStatusService;

impl SystemStatusService {
    pub async fn overview(registry: &ModuleRegistry) -> Result<SystemStatusOverview, ServiceError> {
        let storage = task::spawn_blocking(collect_storage_status).await.map_err(|err| {
            ServiceError::InvalidOperation(format!("Status collection failed: {err}"))
        })?;

        Ok(SystemStatusOverview {
            collected_at: Utc::now(),
            storage,
            modules: collect_module_storage(registry),
            resource: collect_local_resource_status(SystemUtils::get_system_info()),
        })
    }
}

/// Aggregates the module self-reports from the synchronizer snapshot. An
/// available module exposes its current database sizes; an unavailable module
/// hides stale bytes and keeps only the last successful collection time.
fn collect_module_storage(registry: &ModuleRegistry) -> Vec<ModuleDatabaseStatus> {
    let snapshot = registry.snapshot();
    ModuleSpec::fixed()
        .into_iter()
        .map(|spec| {
            let runtime = snapshot.modules().get(spec.id);
            let available = runtime.is_some_and(|item| item.available());
            let report = runtime.and_then(|item| item.storage.as_ref());
            ModuleDatabaseStatus {
                module: spec.id.to_string(),
                available,
                collected_at: report.map(|item| item.collected_at.clone()),
                database: available
                    .then(|| {
                        report.map(|item| SqliteStorageStatus {
                            total_bytes: item.total_bytes,
                            main_bytes: item.main_bytes,
                            wal_bytes: item.wal_bytes,
                            shm_bytes: item.shm_bytes,
                        })
                    })
                    .flatten(),
            }
        })
        .collect()
}

fn collect_storage_status() -> SystemStorageStatus {
    SystemStorageStatus {
        database: collect_sqlite_storage(&CONFIG.admin_database_path()),
        directories: collect_directory_items(),
    }
}

fn collect_sqlite_storage(db_path: &Path) -> SqliteStorageStatus {
    let main_bytes = path_size_bytes(db_path).unwrap_or_else(|err| {
        warn!(
            path = %db_path.display(),
            error = %err,
            "failed to collect sqlite main file size"
        );
        0
    });
    let wal_path = sqlite_sidecar_path(db_path, "-wal");
    let wal_bytes = path_size_bytes(&wal_path).unwrap_or_else(|err| {
        warn!(
            path = %wal_path.display(),
            error = %err,
            "failed to collect sqlite wal file size"
        );
        0
    });
    let shm_path = sqlite_sidecar_path(db_path, "-shm");
    let shm_bytes = path_size_bytes(&shm_path).unwrap_or_else(|err| {
        warn!(
            path = %shm_path.display(),
            error = %err,
            "failed to collect sqlite shm file size"
        );
        0
    });

    SqliteStorageStatus {
        total_bytes: main_bytes.saturating_add(wal_bytes).saturating_add(shm_bytes),
        main_bytes,
        wal_bytes,
        shm_bytes,
    }
}

fn collect_directory_items() -> Vec<DirectoryStorageItem> {
    let runtime_root = CONFIG.runtime_root_dir();
    [
        ("web", "web 静态资源", CONFIG.web_dist_dir()),
        ("bin", "bin 服务程序", runtime_root.join("bin")),
        ("logs", "logs 日志", CONFIG.log_dir()),
        ("data", "data 数据", CONFIG.data_dir()),
    ]
    .into_iter()
    .map(|(key, label, path)| directory_item(key, label, path))
    .collect()
}

fn directory_item(key: &str, label: &str, path: PathBuf) -> DirectoryStorageItem {
    match path_size_bytes(&path) {
        Ok(size_bytes) => DirectoryStorageItem {
            key: key.to_string(),
            label: label.to_string(),
            size_bytes,
            error_message: None,
        },
        Err(err) => {
            warn!(
                path = %path.display(),
                error = %err,
                "failed to collect system status directory size"
            );
            DirectoryStorageItem {
                key: key.to_string(),
                label: label.to_string(),
                size_bytes: 0,
                error_message: Some("Read failed".to_string()),
            }
        }
    }
}

fn collect_local_resource_status(info: SystemInfo) -> LocalResourceStatus {
    LocalResourceStatus {
        cpu: CpuResourceStatus {
            cores: info.cpu_total as u64,
            usage_percent: round_percent(info.cpu_used as f64),
        },
        memory: MemoryResourceStatus {
            total_bytes: info.memory_total,
            used_bytes: info.memory_used,
            available_bytes: info.memory_free,
            usage_percent: percentage(info.memory_used, info.memory_total),
        },
        disk: DiskResourceStatus {
            total_bytes: info.disk_total,
            used_bytes: info.disk_used,
            available_bytes: info.disk_free,
            usage_percent: percentage(info.disk_used, info.disk_total),
        },
    }
}

fn path_size_bytes(path: &Path) -> Result<u64, ServiceError> {
    match path_size_bytes_inner(path) {
        Ok(size) => Ok(size),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(err) => Err(ServiceError::InvalidOperation(format!(
            "Failed to read path size: {}, {}",
            path.display(),
            err
        ))),
    }
}

fn path_size_bytes_inner(path: &Path) -> io::Result<u64> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if metadata.is_symlink() || !metadata.is_dir() {
        return Ok(0);
    }

    let mut total = 0_u64;
    for entry in fs::read_dir(path)? {
        total = total.saturating_add(path_size_bytes_inner(&entry?.path())?);
    }
    Ok(total)
}

fn sqlite_sidecar_path(db_path: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{}", db_path.to_string_lossy(), suffix))
}

fn percentage(used: u64, total: u64) -> f64 {
    if total == 0 { 0.0 } else { round_percent((used as f64 / total as f64) * 100.0) }
}

fn round_percent(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn path_size_counts_nested_files_and_missing_path_as_zero() {
        let dir = std::env::temp_dir().join(format!(
            "rustzen-admin-status-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time after unix epoch")
                .as_nanos()
        ));
        fs::create_dir(&dir).expect("create temp dir");
        let nested = dir.join("nested");
        fs::create_dir(&nested).expect("create nested dir");

        let mut first = fs::File::create(dir.join("first.txt")).expect("create first file");
        first.write_all(b"abc").expect("write first file");
        let mut second = fs::File::create(nested.join("second.txt")).expect("create second file");
        second.write_all(b"defg").expect("write second file");

        assert_eq!(path_size_bytes(&dir).expect("read dir size"), 7);
        assert_eq!(path_size_bytes(&dir.join("missing")).expect("read missing size"), 0);
        fs::remove_dir_all(&dir).expect("remove temp dir");
    }

    #[test]
    fn sqlite_sidecar_path_appends_suffix_to_full_database_filename() {
        let path = PathBuf::from("/tmp/rustzen.db");

        assert_eq!(sqlite_sidecar_path(&path, "-wal"), PathBuf::from("/tmp/rustzen.db-wal"));
        assert_eq!(sqlite_sidecar_path(&path, "-shm"), PathBuf::from("/tmp/rustzen.db-shm"));
    }
}

#[cfg(test)]
mod module_storage_tests {
    use axum::{Json, Router, routing::get};
    use std::{collections::BTreeMap, sync::Arc, time::Duration};
    use tokio::sync::RwLock;

    use crate::features::modules::registry::ModuleRegistry;
    use crate::features::modules::service::ModuleControlState;
    use crate::features::modules::types::ModuleSpec;
    use rustzen_ipc::DelegationSigner;

    use super::SystemStatusService;

    async fn state_with_upstream(
        healthy: bool,
        storage: Option<serde_json::Value>,
    ) -> ModuleControlState {
        let storage = Arc::new(RwLock::new(storage));
        let storage_handler =
            move |State(storage): State<Arc<RwLock<Option<serde_json::Value>>>>| async move {
                match storage.read().await.clone() {
                    Some(value) => Json(value).into_response(),
                    None => StatusCode::SERVICE_UNAVAILABLE.into_response(),
                }
            };
        let upstream = Router::new()
            .route(
                "/health",
                get(move || async move {
                    if healthy {
                        Json(serde_json::json!({ "status": "ok" })).into_response()
                    } else {
                        StatusCode::SERVICE_UNAVAILABLE.into_response()
                    }
                }),
            )
            .route("/internal/v1/storage", get(storage_handler))
            .with_state(Arc::clone(&storage));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let address = listener.local_addr().expect("address");
        tokio::spawn(async move {
            axum::serve(listener, upstream).await.expect("serve");
        });

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        let spec =
            ModuleSpec { id: "monitor", name: "Monitor", base_url: format!("http://{address}") };
        ModuleControlState {
            pool,
            registry: ModuleRegistry::new(vec![spec], &BTreeMap::new()),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(1))
                .build()
                .expect("client"),
            signer: DelegationSigner::new("secret").expect("signer"),
            enabled_update: Arc::default(),
        }
    }

    #[tokio::test]
    async fn available_module_exposes_its_self_reported_database() {
        // 该测试通过 registry 直填 runtime 验证聚合规则，不经网络。
        let state = state_with_upstream(true, None).await;
        state.registry.update_module("monitor", |runtime| {
            runtime.enabled = true;
            runtime.condition = crate::features::modules::types::ModuleCondition::Healthy;
            runtime.storage = Some(rustzen_ipc::ModuleStorageReport {
                module: "monitor".to_string(),
                collected_at: "2026-09-17T12:00:00Z".to_string(),
                total_bytes: 2048,
                main_bytes: 1536,
                wal_bytes: 512,
                shm_bytes: 0,
            });
        });
        let overview = SystemStatusService::overview(&state.registry).await.expect("overview");
        let row = overview.modules.iter().find(|row| row.module == "monitor").expect("monitor row");
        assert!(row.available);
        assert_eq!(row.collected_at.as_deref(), Some("2026-09-17T12:00:00Z"));
        let database = row.database.as_ref().expect("database present");
        assert_eq!(database.total_bytes, 2048);
        assert_eq!(database.main_bytes, 1536);
        assert_eq!(database.wal_bytes, 512);
    }

    #[tokio::test]
    async fn unavailable_module_hides_stale_bytes_but_keeps_the_last_collection_time() {
        let state = state_with_upstream(true, None).await;
        state.registry.update_module("monitor", |runtime| {
            runtime.condition = crate::features::modules::types::ModuleCondition::Unavailable;
            runtime.storage = Some(rustzen_ipc::ModuleStorageReport {
                module: "monitor".to_string(),
                collected_at: "2026-09-17T11:00:00Z".to_string(),
                total_bytes: 999,
                main_bytes: 999,
                wal_bytes: 0,
                shm_bytes: 0,
            });
        });
        let overview = SystemStatusService::overview(&state.registry).await.expect("overview");
        let row = overview.modules.iter().find(|row| row.module == "monitor").expect("monitor row");
        assert!(!row.available);
        assert_eq!(row.collected_at.as_deref(), Some("2026-09-17T11:00:00Z"));
        assert!(row.database.is_none());
    }

    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::IntoResponse;
}
