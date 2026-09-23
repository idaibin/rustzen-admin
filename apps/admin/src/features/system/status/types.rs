use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatusOverview {
    pub collected_at: DateTime<Utc>,
    pub storage: SystemStorageStatus,
    pub modules: Vec<ModuleDatabaseStatus>,
    pub resource: LocalResourceStatus,
}

/// Per-module storage self-report aggregated from the module synchronizer.
/// `database` is present only while the module is available; an unavailable
/// module keeps only its last successful collection time.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleDatabaseStatus {
    pub module: String,
    pub available: bool,
    pub collected_at: Option<String>,
    pub database: Option<SqliteStorageStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SystemStorageStatus {
    pub database: SqliteStorageStatus,
    pub directories: Vec<DirectoryStorageItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SqliteStorageStatus {
    pub total_bytes: u64,
    pub main_bytes: u64,
    pub wal_bytes: u64,
    pub shm_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryStorageItem {
    pub key: String,
    pub label: String,
    pub size_bytes: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LocalResourceStatus {
    pub cpu: CpuResourceStatus,
    pub memory: MemoryResourceStatus,
    pub disk: DiskResourceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CpuResourceStatus {
    pub cores: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MemoryResourceStatus {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DiskResourceStatus {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}
