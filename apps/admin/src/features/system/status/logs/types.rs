use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The only process-log modules exposed by the Admin diagnostics boundary.
pub const MODULE_IDS: [&str; 4] = ["admin", "monitor", "insights", "reports"];

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogListQuery {
    pub module: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogTailQuery {
    pub module: String,
    pub date: String,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogFileSelector {
    pub module: String,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogBackupRequest {
    pub files: Vec<ModuleLogFileSelector>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogCleanupConfirmRequest {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogFileResp {
    pub module: String,
    pub file_name: String,
    pub date: String,
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
    pub readable: bool,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogTailResp {
    pub module: String,
    pub date: String,
    pub content: String,
    pub next_cursor: Option<String>,
    pub truncated: bool,
    pub line_count: usize,
    pub byte_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogCleanupCandidate {
    pub module: String,
    pub file_name: String,
    pub date: String,
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogItemFailure {
    pub module: String,
    pub file_name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogCleanupPreviewResp {
    pub preview_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub cutoff_date: String,
    pub candidates: Vec<ModuleLogCleanupCandidate>,
    pub failures: Vec<ModuleLogItemFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleLogCleanupResultResp {
    pub preview_id: String,
    pub removed: Vec<ModuleLogCleanupCandidate>,
    pub retained: Vec<ModuleLogCleanupCandidate>,
    pub failures: Vec<ModuleLogItemFailure>,
    pub partial: bool,
}
