use std::{
    collections::{BTreeSet, HashMap},
    fs::{self, File, Metadata},
    io,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant, SystemTime},
};

use chrono::{DateTime, Days, NaiveDate, Utc};
use once_cell::sync::Lazy;
use rustzen_auth::auth::CurrentUser;
use rustzen_config::RETENTION_DAYS;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tar::{Builder, Header};
use tokio::task;
use uuid::Uuid;

use crate::{
    common::error::ServiceError,
    features::manage::log::{service::LogService, types::LogWriteCommand},
    infra::config::CONFIG,
};

use super::types::{
    MODULE_IDS, ModuleLogBackupRequest, ModuleLogCleanupCandidate, ModuleLogCleanupConfirmRequest,
    ModuleLogCleanupPreviewResp, ModuleLogCleanupResultResp, ModuleLogFileResp,
    ModuleLogFileSelector, ModuleLogItemFailure, ModuleLogListQuery, ModuleLogTailQuery,
    ModuleLogTailResp,
};

pub const MAX_TAIL_BYTES: usize = 256 * 1024;
pub const MAX_TAIL_LINES: usize = 2_000;
pub const MAX_LINE_BYTES: usize = 16 * 1024;
pub const MAX_ARCHIVE_BYTES: u64 = 64 * 1024 * 1024;

const MAX_ARCHIVE_FILES: usize = 128;
const CLEANUP_PREVIEW_TTL: Duration = Duration::from_secs(60);
const MODULE_LOG_CURSOR_TTL: Duration = Duration::from_secs(60);
const MAX_MODULE_LOG_CURSORS: usize = 256;

static CLEANUP_PREVIEWS: Lazy<Mutex<HashMap<String, CleanupPreviewState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static MODULE_LOG_CURSORS: Lazy<Mutex<HashMap<String, ModuleLogCursorState>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct BackupArchive {
    pub bytes: Vec<u8>,
    pub archive_sha256: String,
    pub file_count: usize,
}

#[derive(Debug, Clone)]
struct ParsedSelector {
    module: String,
    date: NaiveDate,
    file_name: String,
}

#[cfg(unix)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct FileIdentity {
    device: u64,
    inode: u64,
}

#[cfg(not(unix))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct FileIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileSignature {
    size: u64,
    modified: Option<SystemTime>,
    identity: FileIdentity,
}

#[derive(Debug)]
struct FileSnapshot {
    selector: ParsedSelector,
    signature: FileSignature,
    digest: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
struct CleanupCandidateState {
    candidate: ModuleLogCleanupCandidate,
    selector: ParsedSelector,
    signature: FileSignature,
    digest: String,
}

#[derive(Debug, Clone)]
struct CleanupPreviewState {
    preview_id: String,
    expires_at_instant: Instant,
    cutoff_date: NaiveDate,
    candidates: Vec<CleanupCandidateState>,
}

#[derive(Debug, Clone)]
struct ModuleLogCursorState {
    module: String,
    date: NaiveDate,
    file_name: String,
    signature: FileSignature,
    offset: u64,
    expires_at: Instant,
}

pub struct ModuleLogService;

struct CheckedCandidate {
    file: secure_fs::SecureFile,
    selector: ParsedSelector,
    signature: FileSignature,
}

#[cfg(unix)]
#[path = "secure_fs_unix.rs"]
mod secure_fs;
#[cfg(not(unix))]
#[path = "secure_fs_portable.rs"]
mod secure_fs;

mod archive;
mod audit;
mod cleanup;
mod filesystem;
mod listing;
mod operations;
mod selector;

use archive::*;
use audit::*;
use cleanup::*;
use filesystem::*;
use listing::*;
use selector::*;

#[cfg(test)]
mod tests;
