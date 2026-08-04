use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatusOverview {
    pub collected_at: DateTime<Utc>,
    pub storage: SystemStorageStatus,
    pub resource: LocalResourceStatus,
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
