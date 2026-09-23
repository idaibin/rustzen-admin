//! Module storage self-report contract.
//!
//! Every service exposes `GET /internal/v1/storage` beside its Manifest. Like
//! the Manifest endpoint it is unprotected middleware-wise and relies on the
//! internal-host binding. The report is point-in-time file metadata only; no
//! database content is read.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStorageReport {
    pub module: String,
    pub collected_at: String,
    pub total_bytes: u64,
    pub main_bytes: u64,
    pub wal_bytes: u64,
    pub shm_bytes: u64,
}

impl ModuleStorageReport {
    /// Collects the main file plus its `-wal`/`-shm` sidecar sizes. A missing
    /// main file reports zeros, matching the Admin status contract.
    pub fn collect(module: &str, main_path: &Path) -> Self {
        let main_bytes = path_size_bytes(main_path);
        let wal_bytes = path_size_bytes(&sqlite_sidecar_path(main_path, "wal"));
        let shm_bytes = path_size_bytes(&sqlite_sidecar_path(main_path, "shm"));
        Self {
            module: module.to_string(),
            collected_at: Utc::now().to_rfc3339(),
            total_bytes: main_bytes + wal_bytes + shm_bytes,
            main_bytes,
            wal_bytes,
            shm_bytes,
        }
    }
}

fn sqlite_sidecar_path(main_path: &Path, suffix: &str) -> std::path::PathBuf {
    let mut sidecar = main_path.as_os_str().to_os_string();
    sidecar.push(format!("-{suffix}"));
    std::path::PathBuf::from(sidecar)
}

fn path_size_bytes(path: &Path) -> u64 {
    std::fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_reports_main_and_sidecar_bytes_with_zero_for_absent_files() {
        let dir = std::env::temp_dir()
            .join(format!("rz-ipc-storage-{}", std::process::id()))
            .join(Utc::now().timestamp_nanos_opt().unwrap_or_default().to_string());
        std::fs::create_dir_all(&dir).expect("temp dir");
        let main = dir.join("module.db");
        std::fs::write(&main, vec![0u8; 512]).expect("main file");
        std::fs::write(dir.join("module.db-wal"), vec![0u8; 64]).expect("wal file");

        let report = ModuleStorageReport::collect("test", &main);

        assert_eq!(report.module, "test");
        assert_eq!(report.main_bytes, 512);
        assert_eq!(report.wal_bytes, 64);
        assert_eq!(report.shm_bytes, 0);
        assert_eq!(report.total_bytes, 576);
        assert!(!report.collected_at.is_empty());
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn collect_reports_zeros_for_a_missing_database() {
        let report = ModuleStorageReport::collect("test", Path::new("/nonexistent/module.db"));
        assert_eq!((report.main_bytes, report.wal_bytes, report.shm_bytes), (0, 0, 0));
        assert_eq!(report.total_bytes, 0);
    }
}
