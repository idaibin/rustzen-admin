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

#[cfg(unix)]
mod secure_fs {
    use std::{
        ffi::CString,
        fs::File,
        io::{self, Read, Seek, SeekFrom},
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::ffi::OsStrExt,
        },
        path::Path,
    };

    use super::{FileSignature, digest_bytes, same_signature, signature};
    use crate::common::error::ServiceError;
    use uuid::Uuid;

    pub struct SecureDirectory {
        file: File,
    }

    pub struct SecureFile {
        file: File,
    }

    pub fn open_directory(path: &Path) -> Result<Option<SecureDirectory>, ServiceError> {
        let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            ServiceError::InvalidOperation(
                "Configured log directory contains an invalid byte".into(),
            )
        })?;
        let fd = unsafe {
            // SAFETY: `path` is a NUL-terminated CString and the returned descriptor is owned below.
            libc::open(
                path.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if fd < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(ServiceError::InvalidOperation(
                "Configured log directory is unavailable or unsafe".into(),
            ));
        }
        let file = unsafe {
            // SAFETY: `fd` is a valid descriptor returned by `open`; this call transfers its ownership.
            File::from_raw_fd(fd)
        };
        Ok(Some(SecureDirectory { file }))
    }

    impl SecureDirectory {
        pub fn open_file(&self, name: &str) -> Result<SecureFile, ServiceError> {
            let name = component(name)?;
            let fd = unsafe {
                // SAFETY: the directory descriptor and NUL-terminated name are valid for `openat`.
                libc::openat(
                    self.file.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            };
            if fd < 0 {
                return Err(ServiceError::InvalidOperation(
                    "Module log file is unavailable or unsafe".into(),
                ));
            }
            let file = unsafe {
                // SAFETY: `fd` is a valid descriptor returned by `openat`; this call transfers its ownership.
                File::from_raw_fd(fd)
            };
            let metadata = file.metadata().map_err(io_error("stat module log"))?;
            if !metadata.is_file() {
                return Err(ServiceError::InvalidOperation(
                    "Module log file is not a regular file".into(),
                ));
            }
            Ok(SecureFile { file })
        }

        pub fn unlink_if_unchanged(
            &self,
            name: &str,
            expected: FileSignature,
            expected_digest: &str,
        ) -> Result<(), ServiceError> {
            self.unlink_if_unchanged_impl(name, expected, expected_digest, || {})
        }

        #[cfg(test)]
        pub fn unlink_if_unchanged_with_barrier<F>(
            &self,
            name: &str,
            expected: FileSignature,
            expected_digest: &str,
            barrier: F,
        ) -> Result<(), ServiceError>
        where
            F: FnOnce(),
        {
            self.unlink_if_unchanged_impl(name, expected, expected_digest, barrier)
        }

        fn unlink_if_unchanged_impl<F>(
            &self,
            name: &str,
            expected: FileSignature,
            expected_digest: &str,
            barrier: F,
        ) -> Result<(), ServiceError>
        where
            F: FnOnce(),
        {
            let first = self.open_file(name)?;
            let first_signature = first.signature()?;
            if !same_signature(first_signature, expected) {
                return Err(ServiceError::InvalidOperation(
                    "Module log changed since preview".into(),
                ));
            }
            let first_bytes = first.read_all()?;
            if digest_bytes(&first_bytes) != expected_digest
                || !same_signature(first.signature()?, expected)
            {
                return Err(ServiceError::InvalidOperation(
                    "Module log changed since preview".into(),
                ));
            }

            // Re-open relative to the same directory immediately before the transaction. The
            // descriptor identity and digest reject ordinary replacement barriers.
            let current = self.open_file(name)?;
            if !same_signature(current.signature()?, expected)
                || digest_bytes(&current.read_all()?) != expected_digest
            {
                return Err(ServiceError::InvalidOperation(
                    "Module log changed since preview".into(),
                ));
            }

            // Move the entry to a private descriptor-relative name using an atomic no-replace
            // rename. The name is then validated again before unlinking. A same-name replacement
            // racing this point is moved to the private name and fails the identity/digest check;
            // it is restored only with a non-overwriting hard-link transaction.
            barrier();
            let source_name = component(name)?;
            let private_name =
                component(&format!(".rustzen-module-log-{}", Uuid::new_v4().simple()))?;
            rename_noreplace(self.file.as_raw_fd(), source_name.as_ptr(), private_name.as_ptr())?;
            let moved = match self.open_file(private_name.to_str().unwrap_or_default()) {
                Ok(file) => file,
                Err(error) => {
                    self.restore_if_absent(&private_name, &source_name);
                    return Err(error);
                }
            };
            if !same_signature(moved.signature()?, expected)
                || digest_bytes(&moved.read_all()?) != expected_digest
            {
                self.restore_if_absent(&private_name, &source_name);
                return Err(ServiceError::InvalidOperation(
                    "Module log changed during cleanup".into(),
                ));
            }
            let result = unsafe {
                // SAFETY: the directory descriptor and private basename are valid for `unlinkat`.
                libc::unlinkat(self.file.as_raw_fd(), private_name.as_ptr(), 0)
            };
            if result != 0 {
                self.restore_if_absent(&private_name, &source_name);
                return Err(ServiceError::InvalidOperation("Module log cleanup failed".into()));
            }
            if self.exists(&private_name)? {
                return Err(ServiceError::InvalidOperation(
                    "Module log cleanup result could not be verified".into(),
                ));
            }
            Ok(())
        }

        fn restore_if_absent(&self, private_name: &CString, source_name: &CString) {
            let result = unsafe {
                // SAFETY: both names are validated basenames under the owned directory descriptor.
                libc::linkat(
                    self.file.as_raw_fd(),
                    private_name.as_ptr(),
                    self.file.as_raw_fd(),
                    source_name.as_ptr(),
                    0,
                )
            };
            if result == 0 {
                unsafe {
                    // SAFETY: the successful link created a second link to the private entry.
                    libc::unlinkat(self.file.as_raw_fd(), private_name.as_ptr(), 0);
                }
            }
        }

        fn exists(&self, name: &CString) -> Result<bool, ServiceError> {
            let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
            let result = unsafe {
                // SAFETY: `stat` points to writable storage and the descriptor/name remain valid.
                libc::fstatat(
                    self.file.as_raw_fd(),
                    name.as_ptr(),
                    stat.as_mut_ptr(),
                    libc::AT_SYMLINK_NOFOLLOW,
                )
            };
            if result == 0 {
                return Ok(true);
            }
            if io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                Ok(false)
            } else {
                Err(ServiceError::InvalidOperation(
                    "Module log cleanup result could not be verified".into(),
                ))
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn rename_noreplace(
        directory_fd: std::os::fd::RawFd,
        source: *const libc::c_char,
        destination: *const libc::c_char,
    ) -> Result<(), ServiceError> {
        let result = unsafe {
            // SAFETY: both basenames are validated and the directory descriptor is owned by the caller.
            libc::renameat2(directory_fd, source, directory_fd, destination, libc::RENAME_NOREPLACE)
        };
        if result == 0 {
            Ok(())
        } else {
            Err(ServiceError::InvalidOperation(
                "Module log cleanup transaction could not start".into(),
            ))
        }
    }

    #[cfg(target_os = "macos")]
    fn rename_noreplace(
        directory_fd: std::os::fd::RawFd,
        source: *const libc::c_char,
        destination: *const libc::c_char,
    ) -> Result<(), ServiceError> {
        let result = unsafe {
            // SAFETY: both basenames are validated and the directory descriptor is owned by the caller.
            libc::renameatx_np(directory_fd, source, directory_fd, destination, libc::RENAME_EXCL)
        };
        if result == 0 {
            Ok(())
        } else {
            Err(ServiceError::InvalidOperation(
                "Module log cleanup transaction could not start".into(),
            ))
        }
    }

    #[cfg(all(unix, not(any(target_os = "linux", target_os = "macos"))))]
    fn rename_noreplace(
        _: std::os::fd::RawFd,
        _: *const libc::c_char,
        _: *const libc::c_char,
    ) -> Result<(), ServiceError> {
        Err(ServiceError::InvalidOperation(
            "Secure module log cleanup is unsupported on this target".into(),
        ))
    }

    impl SecureFile {
        pub fn signature(&self) -> Result<FileSignature, ServiceError> {
            self.file
                .metadata()
                .map(|metadata| signature(&metadata))
                .map_err(io_error("stat module log"))
        }

        pub fn read_all(&self) -> Result<Vec<u8>, ServiceError> {
            let mut file = self.file.try_clone().map_err(io_error("clone module log"))?;
            file.seek(SeekFrom::Start(0)).map_err(io_error("seek module log"))?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).map_err(io_error("read module log"))?;
            Ok(bytes)
        }

        pub fn read_range(&self, start: u64, length: u64) -> Result<Vec<u8>, ServiceError> {
            let mut file = self.file.try_clone().map_err(io_error("clone module log"))?;
            file.seek(SeekFrom::Start(start)).map_err(io_error("seek module log"))?;
            let mut bytes = Vec::new();
            file.take(length).read_to_end(&mut bytes).map_err(io_error("read module log"))?;
            Ok(bytes)
        }
    }

    fn component(name: &str) -> Result<CString, ServiceError> {
        if name.is_empty()
            || name == "."
            || name == ".."
            || name.contains('/')
            || name.contains('\\')
        {
            return Err(ServiceError::InvalidOperation("Invalid module log file name".into()));
        }
        CString::new(name)
            .map_err(|_| ServiceError::InvalidOperation("Invalid module log file name".into()))
    }

    fn io_error(action: &'static str) -> impl FnOnce(io::Error) -> ServiceError {
        move |error| {
            tracing::warn!(%error, action, "Module log filesystem operation failed");
            ServiceError::InvalidOperation(format!("Module log operation failed: {action}"))
        }
    }
}

#[cfg(not(unix))]
mod secure_fs {
    use std::path::Path;

    use super::{FileSignature, ServiceError};

    pub struct SecureDirectory;
    pub struct SecureFile;

    pub fn open_directory(_: &Path) -> Result<Option<SecureDirectory>, ServiceError> {
        Err(unsupported())
    }

    impl SecureDirectory {
        pub fn open_file(&self, _: &str) -> Result<SecureFile, ServiceError> {
            Err(unsupported())
        }

        pub fn unlink_if_unchanged(
            &self,
            _: &str,
            _: FileSignature,
            _: &str,
        ) -> Result<(), ServiceError> {
            Err(unsupported())
        }
    }

    impl SecureFile {
        pub fn signature(&self) -> Result<FileSignature, ServiceError> {
            Err(unsupported())
        }

        pub fn read_all(&self) -> Result<Vec<u8>, ServiceError> {
            Err(unsupported())
        }

        pub fn read_range(&self, _: u64, _: u64) -> Result<Vec<u8>, ServiceError> {
            Err(unsupported())
        }
    }

    fn unsupported() -> ServiceError {
        ServiceError::InvalidOperation(
            "Secure module log file operations are unsupported on this target".into(),
        )
    }
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

impl ModuleLogService {
    pub async fn list(
        _pool: &SqlitePool,
        _user: &CurrentUser,
        query: ModuleLogListQuery,
    ) -> Result<Vec<ModuleLogFileResp>, ServiceError> {
        let module = query.module.map(|value| parse_module(&value)).transpose()?;
        let date = query.date.map(|value| parse_date(&value)).transpose()?;
        let log_dir = CONFIG.log_dir();
        task::spawn_blocking(move || list_in(&log_dir, module, date)).await.map_err(join_error)?
    }

    pub async fn tail(
        _pool: &SqlitePool,
        _user: &CurrentUser,
        query: ModuleLogTailQuery,
    ) -> Result<ModuleLogTailResp, ServiceError> {
        let selector =
            parse_selector(&ModuleLogFileSelector { module: query.module, date: query.date })?;
        let log_dir = CONFIG.log_dir();
        task::spawn_blocking(move || tail_in(&log_dir, selector, query.cursor.as_deref()))
            .await
            .map_err(join_error)?
    }

    pub async fn backup(
        pool: &SqlitePool,
        user: &CurrentUser,
        request: ModuleLogBackupRequest,
    ) -> Result<BackupArchive, ServiceError> {
        let selection = backup_selection(&request);
        let log_dir = CONFIG.log_dir();
        let archive_result =
            match task::spawn_blocking(move || build_archive(&log_dir, request)).await {
                Ok(result) => result,
                Err(error) => Err(join_error(error)),
            };
        match archive_result {
            Ok(archive) => {
                let file_count = archive.file_count;
                let digest = archive.archive_sha256.clone();
                let mut data =
                    operation_data("success", selection, json!({ "fileCount": file_count }), None);
                if let Some(object) = data.as_object_mut() {
                    object.insert("archiveSha256".into(), json!(digest));
                }
                audit_with_status(
                    pool,
                    user,
                    "MODULE_LOG_BACKUP",
                    "Backed up selected module log files".into(),
                    "SUCCESS",
                    data,
                )
                .await
                .map_err(|_| {
                    ServiceError::InvalidOperation(
                        "Module log backup completed but its audit record could not be persisted"
                            .into(),
                    )
                })?;
                tracing::debug!(archive_sha256 = %digest, file_count, "Module log archive audit persisted");
                Ok(archive)
            }
            Err(error) => {
                let data = operation_data(
                    "failure",
                    selection,
                    json!({ "fileCount": 0 }),
                    Some("backup_failed"),
                );
                audit_failure(
                    pool,
                    user,
                    "MODULE_LOG_BACKUP",
                    "Module log backup failed".into(),
                    data,
                )
                .await;
                Err(error)
            }
        }
    }

    pub async fn preview_cleanup(
        pool: &SqlitePool,
        user: &CurrentUser,
    ) -> Result<ModuleLogCleanupPreviewResp, ServiceError> {
        let today = Utc::now().date_naive();
        let cutoff_date = today - Days::new(RETENTION_DAYS);
        let log_dir = CONFIG.log_dir();
        let collection = match task::spawn_blocking(move || {
            collect_cleanup_candidates(&log_dir, today, cutoff_date)
        })
        .await
        {
            Ok(result) => result,
            Err(error) => Err(join_error(error)),
        };
        let (candidates, failures) = match collection {
            Ok(result) => result,
            Err(error) => {
                audit_failure(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_PREVIEW",
                    "Module log cleanup preview failed".into(),
                    operation_data(
                        "failure",
                        json!({ "cutoffDate": cutoff_date.to_string() }),
                        json!({ "candidateCount": 0, "failureCount": 1 }),
                        Some("preview_failed"),
                    ),
                )
                .await;
                return Err(error);
            }
        };

        let preview_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let expires_at =
            Utc::now() + chrono::Duration::seconds(CLEANUP_PREVIEW_TTL.as_secs() as i64);
        let state = CleanupPreviewState {
            preview_id: preview_id.clone(),
            expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
            cutoff_date,
            candidates: candidates.clone(),
        };
        if let Err(error) = store_preview(token.clone(), state) {
            audit_failure(
                pool,
                user,
                "MODULE_LOG_CLEANUP_PREVIEW",
                "Module log cleanup preview could not be stored".into(),
                operation_data(
                    "failure",
                    json!({ "previewId": preview_id, "cutoffDate": cutoff_date.to_string() }),
                    json!({ "candidateCount": candidates.len(), "failureCount": failures.len() }),
                    Some("preview_state_unavailable"),
                ),
            )
            .await;
            return Err(error);
        }

        let status = if failures.is_empty() { "SUCCESS" } else { "PARTIAL" };
        if audit_with_status(
            pool,
            user,
            "MODULE_LOG_CLEANUP_PREVIEW",
            "Previewed module log cleanup candidates".into(),
            status,
            operation_data(
                "success",
                json!({
                    "previewId": preview_id,
                    "cutoffDate": cutoff_date.to_string(),
                    "files": candidates.iter().map(|state| &state.candidate).collect::<Vec<_>>()
                }),
                json!({
                    "candidateCount": candidates.len(),
                    "failureCount": failures.len()
                }),
                (!failures.is_empty()).then_some("candidate_unavailable"),
            ),
        )
        .await
        .is_err()
        {
            remove_preview(&token);
            return Err(ServiceError::InvalidOperation(
                "Module log cleanup preview could not be audited".into(),
            ));
        }

        Ok(ModuleLogCleanupPreviewResp {
            preview_id,
            token,
            expires_at,
            cutoff_date: cutoff_date.to_string(),
            candidates: candidates.into_iter().map(|state| state.candidate).collect(),
            failures,
        })
    }

    pub async fn confirm_cleanup(
        pool: &SqlitePool,
        user: &CurrentUser,
        request: ModuleLogCleanupConfirmRequest,
    ) -> Result<ModuleLogCleanupResultResp, ServiceError> {
        let preview = match take_preview(&request.token) {
            Ok(preview) => preview,
            Err(error) => {
                audit_failure(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_CONFIRM",
                    "Module log cleanup confirmation was rejected".into(),
                    operation_data(
                        "failure",
                        json!({ "previewId": serde_json::Value::Null }),
                        json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 1 }),
                        Some("invalid_confirmation"),
                    ),
                )
                .await;
                return Err(error);
            }
        };
        let intent_data = operation_data(
            "intent",
            cleanup_selection(&preview),
            json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 0 }),
            None,
        );
        let intent_id = audit_with_status(
            pool,
            user,
            "MODULE_LOG_CLEANUP_CONFIRM",
            "Persisted module log cleanup intent".into(),
            "PENDING",
            intent_data,
        )
        .await
        .map_err(|_| {
            ServiceError::InvalidOperation(
                "Module log cleanup intent could not be persisted; no files were removed".into(),
            )
        })?;
        let log_dir = CONFIG.log_dir();
        let preview_id = preview.preview_id.clone();
        let selection = cleanup_selection(&preview);
        let today = Utc::now().date_naive();
        let execution_preview = preview.clone();
        let execution = match task::spawn_blocking(move || {
            execute_cleanup(&log_dir, &execution_preview, today)
        })
        .await
        {
            Ok(result) => result,
            Err(error) => Err(join_error(error)),
        };
        match execution {
            Ok(result) => {
                let status = if result.partial { "PARTIAL" } else { "SUCCESS" };
                let failure_category = (!result.failures.is_empty()).then_some("item_failed");
                let result_data = operation_data(
                    "result",
                    json!({
                        "previewId": preview_id,
                        "intentId": intent_id,
                        "files": selection["files"].clone()
                    }),
                    json!({
                        "removedCount": result.removed.len(),
                        "retainedCount": result.retained.len(),
                        "failureCount": result.failures.len(),
                        "partial": result.partial
                    }),
                    failure_category,
                );
                if audit_with_status(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_CONFIRM",
                    "Persisted module log cleanup result".into(),
                    status,
                    result_data,
                )
                .await
                .is_err()
                {
                    return Err(ServiceError::InvalidOperation(format!(
                        "Module log cleanup result could not be persisted after intent {intent_id}; outcome is partial or unknown and the intent remains durable"
                    )));
                }
                Ok(result)
            }
            Err(error) => {
                let result_audit = audit_with_status(
                    pool,
                    user,
                    "MODULE_LOG_CLEANUP_CONFIRM",
                    "Module log cleanup execution failed".into(),
                    "FAILED",
                    operation_data(
                        "result",
                        json!({ "previewId": preview_id, "intentId": intent_id }),
                        json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 1 }),
                        Some("cleanup_execution_failed"),
                    ),
                )
                .await;
                if result_audit.is_err() {
                    return Err(ServiceError::InvalidOperation(format!(
                        "Module log cleanup failed and its result could not be persisted; intent {intent_id} remains durable"
                    )));
                }
                Err(error)
            }
        }
    }
}

fn list_in(
    log_dir: &Path,
    module_filter: Option<&str>,
    date_filter: Option<NaiveDate>,
) -> Result<Vec<ModuleLogFileResp>, ServiceError> {
    let Some(root) = checked_log_root(log_dir)? else {
        return Ok(Vec::new());
    };
    let today = Utc::now().date_naive();
    let mut items = Vec::new();
    for entry in fs::read_dir(&root).map_err(io_error("read log directory"))? {
        let entry = entry.map_err(io_error("read log directory entry"))?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some((module, date)) = parse_file_name(&file_name) else {
            continue;
        };
        if module_filter.is_some_and(|filter| filter != module)
            || date_filter.is_some_and(|filter| filter != date)
        {
            continue;
        }
        let path = root.join(&file_name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let is_symlink = metadata.file_type().is_symlink();
        let readable = !is_symlink && metadata.is_file() && File::open(&path).is_ok();
        let modified_at =
            metadata.modified().map(DateTime::<Utc>::from).unwrap_or_else(|_| Utc::now());
        items.push(ModuleLogFileResp {
            module: module.to_string(),
            file_name,
            date: date.to_string(),
            size_bytes: if readable { metadata.len() } else { 0 },
            modified_at,
            readable,
            active: date == today,
        });
    }
    items.sort_by(|left, right| {
        left.module.cmp(&right.module).then_with(|| right.date.cmp(&left.date))
    });
    Ok(items)
}

fn tail_in(
    log_dir: &Path,
    selector: ParsedSelector,
    cursor: Option<&str>,
) -> Result<ModuleLogTailResp, ServiceError> {
    let checked = checked_candidate(log_dir, &selector)?;
    let end = match cursor {
        Some(cursor) => decode_cursor(cursor, &selector, checked.signature)?,
        None => checked.signature.size,
    };
    let start = end.saturating_sub((MAX_TAIL_BYTES + MAX_LINE_BYTES) as u64);
    let bytes = checked.file.read_range(start, end.saturating_sub(start))?;
    if !same_signature(checked.file.signature()?, checked.signature) {
        return Err(ServiceError::InvalidOperation("Module log changed while it was read".into()));
    }

    if !bytes.is_empty() && !bytes.contains(&b'\n') {
        return tail_single_line_chunk(&selector, checked.signature, start, end, &bytes);
    }

    let mut truncated = start > 0;
    let mut content_start = start;
    let mut content_bytes = bytes.as_slice();
    if start > 0 {
        if let Some(index) = bytes.iter().position(|byte| *byte == b'\n') {
            content_start = start + index as u64 + 1;
            content_bytes = &bytes[index + 1..];
        } else {
            content_start = end;
            content_bytes = &[];
        }
    }

    let mut lines = Vec::new();
    let mut line_offset = content_start;
    for chunk in content_bytes.split_inclusive(|byte| *byte == b'\n') {
        let line_bytes = chunk.strip_suffix(b"\n").unwrap_or(chunk);
        lines.push((line_offset, String::from_utf8_lossy(line_bytes).into_owned()));
        line_offset += chunk.len() as u64;
    }
    if lines.len() > MAX_TAIL_LINES {
        let drop_count = lines.len() - MAX_TAIL_LINES;
        lines.drain(0..drop_count);
        truncated = true;
    }
    let mut bounded = Vec::with_capacity(lines.len());
    for (offset, line) in lines {
        let (line, line_truncated) = truncate_utf8(&line, MAX_LINE_BYTES);
        truncated |= line_truncated;
        bounded.push((offset, line));
    }
    while bounded.iter().map(|(_, line)| line.len()).sum::<usize>()
        + bounded.len().saturating_sub(1)
        > MAX_TAIL_BYTES
    {
        if bounded.is_empty() {
            break;
        }
        bounded.remove(0);
        truncated = true;
    }
    let next_offset = bounded.first().map(|(offset, _)| *offset).filter(|offset| *offset > 0);
    let content = bounded.iter().map(|(_, line)| line.as_str()).collect::<Vec<_>>().join("\n");
    let next_cursor = next_offset
        .map(|offset| encode_cursor(&selector, checked.signature, offset))
        .transpose()?;
    Ok(ModuleLogTailResp {
        module: selector.module,
        date: selector.date.to_string(),
        line_count: bounded.len(),
        byte_count: content.len(),
        content,
        next_cursor,
        truncated,
    })
}

fn tail_single_line_chunk(
    selector: &ParsedSelector,
    signature: FileSignature,
    start: u64,
    end: u64,
    bytes: &[u8],
) -> Result<ModuleLogTailResp, ServiceError> {
    let segment_start = start.max(end.saturating_sub(MAX_LINE_BYTES as u64));
    let relative_start = segment_start.saturating_sub(start) as usize;
    let segment = &bytes[relative_start..];
    let (content, line_truncated) =
        truncate_utf8(&String::from_utf8_lossy(segment), MAX_LINE_BYTES);
    let next_cursor = (segment_start > 0)
        .then(|| encode_cursor(selector, signature, segment_start))
        .transpose()?;
    Ok(ModuleLogTailResp {
        module: selector.module.clone(),
        date: selector.date.to_string(),
        line_count: 1,
        byte_count: content.len(),
        content,
        next_cursor,
        truncated: line_truncated
            || start > 0
            || segment.len() < bytes.len()
            || end < signature.size,
    })
}

fn build_archive(
    log_dir: &Path,
    request: ModuleLogBackupRequest,
) -> Result<BackupArchive, ServiceError> {
    if request.files.is_empty() {
        return Err(ServiceError::InvalidOperation(
            "At least one module log file is required".into(),
        ));
    }
    if request.files.len() > MAX_ARCHIVE_FILES {
        return Err(ServiceError::InvalidOperation(
            "Too many module log files were selected".into(),
        ));
    }

    let mut selectors = BTreeSet::new();
    let mut parsed = Vec::new();
    for selector in request.files {
        let selector_key = format!("{}:{}", selector.module, selector.date);
        if !selectors.insert(selector_key) {
            continue;
        }
        parsed.push(parse_selector(&selector)?);
    }

    let mut snapshots = Vec::with_capacity(parsed.len());
    let mut expected_bytes = 1_024_u64;
    for selector in parsed {
        let checked = checked_candidate(log_dir, &selector)?;
        expected_bytes = expected_bytes.saturating_add(tar_entry_size(checked.signature.size));
        if expected_bytes > MAX_ARCHIVE_BYTES {
            return Err(ServiceError::InvalidOperation(
                "The selected archive exceeds 64 MiB".into(),
            ));
        }
        let snapshot = read_snapshot(&checked)?;
        snapshots.push(snapshot);
    }
    let manifest = manifest_bytes(&snapshots)?;
    expected_bytes = expected_bytes.saturating_add(tar_entry_size(manifest.len() as u64));
    if expected_bytes > MAX_ARCHIVE_BYTES {
        return Err(ServiceError::InvalidOperation("The selected archive exceeds 64 MiB".into()));
    }

    let mut archive = Vec::with_capacity(expected_bytes as usize);
    {
        let mut builder = Builder::new(&mut archive);
        for snapshot in &snapshots {
            let checked = checked_candidate(log_dir, &snapshot.selector)?;
            let current = read_snapshot(&checked)?;
            if current.signature != snapshot.signature || current.digest != snapshot.digest {
                return Err(ServiceError::InvalidOperation(
                    "A selected module log changed while the archive was being built".into(),
                ));
            }
            append_tar_entry(&mut builder, &snapshot.selector.file_name, &current.bytes)?;
        }
        append_tar_entry(&mut builder, "manifest.json", &manifest)?;
        builder.finish().map_err(io_error("finish module log archive"))?;
    }
    if archive.len() as u64 > MAX_ARCHIVE_BYTES {
        return Err(ServiceError::InvalidOperation("The selected archive exceeds 64 MiB".into()));
    }
    let archive_sha256 = digest_bytes(&archive);
    Ok(BackupArchive { bytes: archive, archive_sha256, file_count: snapshots.len() })
}

#[derive(Debug, Serialize)]
struct ArchiveManifestEntry<'a> {
    module: &'a str,
    file_name: &'a str,
    date: String,
    size_bytes: u64,
    modified_at: DateTime<Utc>,
    sha256: &'a str,
}

fn manifest_bytes(snapshots: &[FileSnapshot]) -> Result<Vec<u8>, ServiceError> {
    let entries = snapshots
        .iter()
        .map(|snapshot| ArchiveManifestEntry {
            module: &snapshot.selector.module,
            file_name: &snapshot.selector.file_name,
            date: snapshot.selector.date.to_string(),
            size_bytes: snapshot.signature.size,
            modified_at: snapshot
                .signature
                .modified
                .map(DateTime::<Utc>::from)
                .unwrap_or_else(Utc::now),
            sha256: &snapshot.digest,
        })
        .collect::<Vec<_>>();
    serde_json::to_vec_pretty(&entries).map_err(|error| {
        ServiceError::InvalidOperation(format!("Failed to build archive manifest: {error}"))
    })
}

fn append_tar_entry(
    builder: &mut Builder<&mut Vec<u8>>,
    name: &str,
    bytes: &[u8],
) -> Result<(), ServiceError> {
    let mut header = Header::new_gnu();
    header.set_path(name).map_err(|error| {
        ServiceError::InvalidOperation(format!("Invalid archive entry name: {error}"))
    })?;
    header.set_size(bytes.len() as u64);
    header.set_mode(0o600);
    header.set_cksum();
    builder.append(&header, bytes).map_err(io_error("append module log archive entry"))
}

fn collect_cleanup_candidates(
    log_dir: &Path,
    today: NaiveDate,
    cutoff_date: NaiveDate,
) -> Result<(Vec<CleanupCandidateState>, Vec<ModuleLogItemFailure>), ServiceError> {
    let Some(directory) = secure_fs::open_directory(log_dir)? else {
        return Ok((Vec::new(), Vec::new()));
    };
    let Some(root) = checked_log_root(log_dir)? else {
        return Ok((Vec::new(), Vec::new()));
    };
    let mut candidates = Vec::new();
    let mut failures = Vec::new();
    for entry in fs::read_dir(&root).map_err(io_error("read log directory"))? {
        let entry = entry.map_err(io_error("read log directory entry"))?;
        let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some((module, date)) = parse_file_name(&file_name) else {
            continue;
        };
        if date >= cutoff_date || date >= today {
            continue;
        }
        let selector =
            ParsedSelector { module: module.to_string(), date, file_name: file_name.clone() };
        let file = match directory.open_file(&selector.file_name) {
            Ok(file) => file,
            Err(_) => {
                failures.push(ModuleLogItemFailure {
                    module: selector.module,
                    file_name,
                    reason: "file changed or is unsafe".into(),
                });
                continue;
            }
        };
        let signature = file.signature()?;
        let bytes = match file.read_all() {
            Ok(bytes) => bytes,
            Err(_) => {
                failures.push(ModuleLogItemFailure {
                    module: selector.module,
                    file_name,
                    reason: "file unavailable".into(),
                });
                continue;
            }
        };
        if !same_signature(signature, file.signature()?) {
            failures.push(ModuleLogItemFailure {
                module: selector.module,
                file_name,
                reason: "file changed while it was read".into(),
            });
            continue;
        }
        let modified_at = signature.modified.map(DateTime::<Utc>::from).unwrap_or_else(Utc::now);
        candidates.push(CleanupCandidateState {
            candidate: ModuleLogCleanupCandidate {
                module: selector.module.clone(),
                file_name,
                date: selector.date.to_string(),
                size_bytes: signature.size,
                modified_at,
            },
            selector,
            signature,
            digest: digest_bytes(&bytes),
        });
    }
    candidates.sort_by(|left, right| left.candidate.file_name.cmp(&right.candidate.file_name));
    Ok((candidates, failures))
}

fn execute_cleanup(
    log_dir: &Path,
    preview: &CleanupPreviewState,
    today: NaiveDate,
) -> Result<ModuleLogCleanupResultResp, ServiceError> {
    let mut removed = Vec::new();
    let mut retained = Vec::new();
    let mut failures = Vec::new();
    let directory = secure_fs::open_directory(log_dir)?.ok_or_else(|| {
        ServiceError::InvalidOperation("Module log directory is unavailable".into())
    })?;
    for state in &preview.candidates {
        let candidate = state.candidate.clone();
        if state.selector.date >= today || state.selector.date >= preview.cutoff_date {
            retained.push(candidate);
            continue;
        }
        match directory.unlink_if_unchanged(
            &state.selector.file_name,
            state.signature,
            &state.digest,
        ) {
            Ok(()) => removed.push(candidate),
            Err(_) => failures.push(ModuleLogItemFailure {
                module: candidate.module,
                file_name: candidate.file_name,
                reason: "file changed or cleanup failed".into(),
            }),
        }
    }
    Ok(ModuleLogCleanupResultResp {
        preview_id: preview.preview_id.clone(),
        partial: !retained.is_empty() || !failures.is_empty(),
        removed,
        retained,
        failures,
    })
}

fn checked_log_root(log_dir: &Path) -> Result<Option<PathBuf>, ServiceError> {
    let metadata = match fs::symlink_metadata(log_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_error("stat log directory")(error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ServiceError::InvalidOperation("Configured log directory is unsafe".into()));
    }
    fs::canonicalize(log_dir).map(Some).map_err(io_error("resolve log directory"))
}

struct CheckedCandidate {
    file: secure_fs::SecureFile,
    selector: ParsedSelector,
    signature: FileSignature,
}

fn checked_candidate(
    log_dir: &Path,
    selector: &ParsedSelector,
) -> Result<CheckedCandidate, ServiceError> {
    let directory = secure_fs::open_directory(log_dir)?.ok_or_else(|| {
        ServiceError::InvalidOperation("Module log directory is unavailable".into())
    })?;
    let file = directory.open_file(&selector.file_name)?;
    let signature = file.signature()?;
    Ok(CheckedCandidate { file, selector: selector.clone(), signature })
}

fn read_snapshot(checked: &CheckedCandidate) -> Result<FileSnapshot, ServiceError> {
    let bytes = checked.file.read_all()?;
    if !same_signature(checked.signature, checked.file.signature()?) {
        return Err(ServiceError::InvalidOperation("Module log changed while it was read".into()));
    }
    Ok(FileSnapshot {
        selector: checked.selector.clone(),
        signature: checked.signature,
        digest: digest_bytes(&bytes),
        bytes,
    })
}

fn parse_selector(selector: &ModuleLogFileSelector) -> Result<ParsedSelector, ServiceError> {
    let module = parse_module(&selector.module)?;
    let date = parse_date(&selector.date)?;
    Ok(ParsedSelector { module: module.to_owned(), date, file_name: format!("{module}.{date}") })
}

fn parse_module(value: &str) -> Result<&'static str, ServiceError> {
    MODULE_IDS
        .iter()
        .copied()
        .find(|module| *module == value)
        .ok_or_else(|| ServiceError::InvalidOperation("Unknown module log selector".into()))
}

fn parse_date(value: &str) -> Result<NaiveDate, ServiceError> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ServiceError::InvalidOperation("Invalid module log date".into()))?;
    if date.format("%Y-%m-%d").to_string() != value {
        return Err(ServiceError::InvalidOperation("Invalid module log date".into()));
    }
    Ok(date)
}

fn parse_file_name(value: &str) -> Option<(&'static str, NaiveDate)> {
    let (module, raw_date) = value.split_once('.')?;
    if value.matches('.').count() != 1 {
        return None;
    }
    let module = MODULE_IDS.iter().copied().find(|candidate| *candidate == module)?;
    let date = NaiveDate::parse_from_str(raw_date, "%Y-%m-%d").ok()?;
    (date.format("%Y-%m-%d").to_string() == raw_date).then_some((module, date))
}

fn signature(metadata: &Metadata) -> FileSignature {
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        FileIdentity { device: metadata.dev(), inode: metadata.ino() }
    };
    #[cfg(not(unix))]
    let identity = FileIdentity;
    FileSignature { size: metadata.len(), modified: metadata.modified().ok(), identity }
}

fn same_signature(left: FileSignature, right: FileSignature) -> bool {
    left == right
}

fn digest_bytes(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    hex::encode(digest.finalize())
}

fn tar_entry_size(size: u64) -> u64 {
    512_u64.saturating_add(size.div_ceil(512).saturating_mul(512))
}

fn truncate_utf8(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_owned(), false);
    }
    let mut end = max_bytes;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_owned(), true)
}

fn encode_cursor(
    selector: &ParsedSelector,
    signature: FileSignature,
    offset: u64,
) -> Result<String, ServiceError> {
    let token = Uuid::new_v4().simple().to_string();
    let mut cursors = MODULE_LOG_CURSORS.lock().map_err(|_| {
        ServiceError::InvalidOperation("Module log cursor state is unavailable".into())
    })?;
    let now = Instant::now();
    cursors.retain(|_, state| state.expires_at > now);
    if cursors.len() >= MAX_MODULE_LOG_CURSORS
        && let Some((oldest, _)) = cursors.iter().min_by_key(|(_, state)| state.expires_at)
    {
        let oldest = oldest.clone();
        cursors.remove(&oldest);
    }
    cursors.insert(
        token.clone(),
        ModuleLogCursorState {
            module: selector.module.clone(),
            date: selector.date,
            file_name: selector.file_name.clone(),
            signature,
            offset,
            expires_at: now + MODULE_LOG_CURSOR_TTL,
        },
    );
    Ok(token)
}

fn decode_cursor(
    cursor: &str,
    selector: &ParsedSelector,
    signature: FileSignature,
) -> Result<u64, ServiceError> {
    let mut cursors = MODULE_LOG_CURSORS.lock().map_err(|_| {
        ServiceError::InvalidOperation("Module log cursor state is unavailable".into())
    })?;
    let now = Instant::now();
    cursors.retain(|_, state| state.expires_at > now);
    let state = cursors.get(cursor).ok_or_else(|| {
        ServiceError::InvalidOperation("Invalid or expired module log cursor".into())
    })?;
    if state.module != selector.module
        || state.date != selector.date
        || state.file_name != selector.file_name
        || state.signature != signature
        || state.offset > signature.size
    {
        return Err(ServiceError::InvalidOperation(
            "Module log cursor does not match the selected file".into(),
        ));
    }
    Ok(state.offset)
}

fn store_preview(token: String, state: CleanupPreviewState) -> Result<(), ServiceError> {
    let mut previews = CLEANUP_PREVIEWS.lock().map_err(|_| {
        ServiceError::InvalidOperation("Cleanup confirmation is unavailable".into())
    })?;
    let now = Instant::now();
    previews.retain(|_, preview| preview.expires_at_instant > now);
    previews.insert(token, state);
    Ok(())
}

fn remove_preview(token: &str) {
    if let Ok(mut previews) = CLEANUP_PREVIEWS.lock() {
        previews.remove(token);
    }
}

fn take_preview(token: &str) -> Result<CleanupPreviewState, ServiceError> {
    let mut previews = CLEANUP_PREVIEWS.lock().map_err(|_| {
        ServiceError::InvalidOperation("Cleanup confirmation is unavailable".into())
    })?;
    let preview = previews.remove(token).ok_or_else(|| {
        ServiceError::InvalidOperation("Cleanup confirmation is invalid or expired".into())
    })?;
    if preview.expires_at_instant <= Instant::now() {
        return Err(ServiceError::InvalidOperation(
            "Cleanup confirmation is invalid or expired".into(),
        ));
    }
    Ok(preview)
}

fn backup_selection(request: &ModuleLogBackupRequest) -> serde_json::Value {
    json!({
        "files": request.files.iter().map(|file| json!({
            "module": file.module,
            "date": file.date
        })).collect::<Vec<_>>()
    })
}

fn cleanup_selection(preview: &CleanupPreviewState) -> serde_json::Value {
    json!({
        "previewId": preview.preview_id,
        "cutoffDate": preview.cutoff_date.to_string(),
        "files": preview.candidates.iter().map(|state| &state.candidate).collect::<Vec<_>>()
    })
}

fn operation_data(
    phase: &str,
    selection: serde_json::Value,
    result_counts: serde_json::Value,
    failure_category: Option<&str>,
) -> serde_json::Value {
    json!({
        "phase": phase,
        "selection": selection,
        "resultCounts": result_counts,
        "failureCategory": failure_category
    })
}

async fn audit_with_status(
    pool: &SqlitePool,
    user: &CurrentUser,
    action: &'static str,
    description: String,
    status: &'static str,
    data: serde_json::Value,
) -> Result<i64, ServiceError> {
    let command = LogWriteCommand {
        user_id: user.user_id,
        username: user.username.clone(),
        action: action.to_owned(),
        description,
        data: Some(data),
        status: status.into(),
        duration_ms: 0,
        ip_address: "module-log".into(),
        user_agent: "Admin module-log diagnostics".into(),
    };
    LogService::record_operation(pool, command).await.map_err(|error| {
        tracing::error!(%error, action, status, "Failed to audit module log operation");
        error
    })
}

async fn audit_failure(
    pool: &SqlitePool,
    user: &CurrentUser,
    action: &'static str,
    description: String,
    data: serde_json::Value,
) {
    if let Err(error) = audit_with_status(pool, user, action, description, "FAILED", data).await {
        tracing::error!(%error, action, "Failed to persist module log failure audit");
    }
}

fn io_error(action: &'static str) -> impl FnOnce(io::Error) -> ServiceError {
    move |error| {
        tracing::warn!(%error, action, "Module log filesystem operation failed");
        ServiceError::InvalidOperation(format!("Module log operation failed: {action}"))
    }
}

fn join_error(error: task::JoinError) -> ServiceError {
    ServiceError::InvalidOperation(format!("Module log operation failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::{
        fs::{self, OpenOptions},
        io::Write,
        path::Path,
    };

    fn temp_log_dir() -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("rustzen-admin-module-logs-{}", Uuid::new_v4()));
        fs::create_dir(&path).unwrap();
        path
    }

    fn selector(module: &str, date: &str, _root: &Path) -> ParsedSelector {
        ParsedSelector {
            module: module.into(),
            date: NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
            file_name: format!("{module}.{date}"),
        }
    }

    #[test]
    fn selector_rejects_unknown_modules_traversal_and_noncanonical_dates() {
        assert!(parse_module("../../etc").is_err());
        assert!(parse_date("2026-1-01").is_err());
        assert!(
            parse_selector(&ModuleLogFileSelector {
                module: "admin/../monitor".into(),
                date: "2026-01-01".into(),
            })
            .is_err()
        );
    }

    #[test]
    fn file_name_parser_compares_the_raw_date_to_the_canonical_form() {
        assert!(parse_file_name("admin.2026-1-01").is_none());
        assert!(parse_file_name("admin.2026-01-1").is_none());
        assert!(parse_file_name("admin.2026-01-01.extra").is_none());
        assert_eq!(
            parse_file_name("admin.2026-01-01").map(|(_, date)| date),
            NaiveDate::from_ymd_opt(2026, 1, 1)
        );
    }

    #[test]
    fn cursor_is_opaque_and_bound_to_file_signature_and_selector() {
        let root = Path::new("/var/lib/rustzen/logs");
        let selected = selector("admin", "2026-01-01", root);
        let signature =
            FileSignature { size: 128, modified: None, identity: FileIdentity::default() };
        let token = encode_cursor(&selected, signature, 64).unwrap();
        assert!(!token.starts_with("r1:"));
        assert!(!token.contains("0000000000000040"));
        assert_eq!(decode_cursor(&token, &selected, signature).unwrap(), 64);

        let other_file = selector("monitor", "2026-01-01", root);
        assert!(decode_cursor(&token, &other_file, signature).is_err());
        assert!(
            decode_cursor(&token, &selected, FileSignature { size: 129, ..signature },).is_err()
        );
        assert!(decode_cursor("r1:0000000000000040", &selected, signature).is_err());
    }

    #[test]
    fn list_marks_symlink_unreadable_without_following_target() {
        let dir = temp_log_dir();
        fs::write(dir.join("admin.2026-01-01"), b"safe").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("admin.2026-01-01", dir.join("monitor.2026-01-01")).unwrap();
        let items = list_in(&dir, None, None).unwrap();
        assert!(items.iter().any(|item| item.module == "admin" && item.readable));
        #[cfg(unix)]
        assert!(items.iter().any(|item| item.module == "monitor" && !item.readable));
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn tail_is_bounded_by_bytes_lines_and_individual_line_size() {
        let dir = temp_log_dir();
        let mut file = fs::File::create(dir.join("admin.2026-01-01")).unwrap();
        for index in 0..(MAX_TAIL_LINES + 20) {
            writeln!(file, "{index}:{}", "x".repeat(MAX_LINE_BYTES + 100)).unwrap();
        }
        let result = tail_in(&dir, selector("admin", "2026-01-01", &dir), None).unwrap();
        assert!(result.truncated);
        assert!(result.byte_count <= MAX_TAIL_BYTES);
        assert!(result.line_count <= MAX_TAIL_LINES);
        assert!(result.content.lines().all(|line| line.len() <= MAX_LINE_BYTES));
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn tail_paginates_10k_short_lines_without_gaps_or_duplicates() {
        let dir = temp_log_dir();
        let mut file = fs::File::create(dir.join("admin.2026-01-01")).unwrap();
        let expected = (0..10_000).map(|index| format!("line-{index:05}")).collect::<Vec<_>>();
        for line in &expected {
            writeln!(file, "{line}").unwrap();
        }

        let mut cursor = None;
        let mut pages = Vec::new();
        loop {
            let page =
                tail_in(&dir, selector("admin", "2026-01-01", &dir), cursor.as_deref()).unwrap();
            assert!(!page.content.is_empty());
            pages.extend(page.content.lines().map(str::to_owned));
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        assert_eq!(pages.len(), expected.len());
        pages.sort();
        assert_eq!(pages, expected);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn tail_paginates_a_single_unterminated_line_larger_than_the_read_window() {
        let dir = temp_log_dir();
        let expected = "x".repeat(MAX_TAIL_BYTES + MAX_LINE_BYTES + 123);
        fs::write(dir.join("admin.2026-01-01"), &expected).unwrap();

        let mut cursor = None;
        let mut chunks = Vec::new();
        loop {
            let page =
                tail_in(&dir, selector("admin", "2026-01-01", &dir), cursor.as_deref()).unwrap();
            assert_eq!(page.line_count, 1);
            assert!(!page.content.is_empty());
            assert!(page.byte_count <= MAX_LINE_BYTES);
            assert!(page.truncated);
            chunks.push(page.content);
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        let reconstructed = chunks.into_iter().rev().collect::<String>();
        assert_eq!(reconstructed, expected);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn backup_contains_manifest_hash_and_respects_archive_cap() {
        let dir = temp_log_dir();
        fs::write(dir.join("admin.2026-01-01"), b"hello").unwrap();
        let request = ModuleLogBackupRequest {
            files: vec![ModuleLogFileSelector {
                module: "admin".into(),
                date: "2026-01-01".into(),
            }],
        };
        let archive = build_archive(&dir, request).unwrap();
        assert!(
            archive.bytes.windows(b"manifest.json".len()).any(|window| window == b"manifest.json")
        );
        assert_eq!(archive.archive_sha256, digest_bytes(&archive.bytes));
        assert!(archive.bytes.len() as u64 <= MAX_ARCHIVE_BYTES);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_preview_excludes_today_and_confirmation_is_single_use() {
        let dir = temp_log_dir();
        let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let cutoff = today - Days::new(RETENTION_DAYS);
        fs::write(dir.join("admin.2026-06-01"), b"old").unwrap();
        fs::write(dir.join("admin.2026-08-10"), b"today").unwrap();
        fs::write(dir.join("admin.2026-07-15"), b"inside").unwrap();
        let (candidates, failures) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
        assert!(failures.is_empty());
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].candidate.date, "2026-06-01");

        let token = "test-token".to_string();
        store_preview(
            token.clone(),
            CleanupPreviewState {
                preview_id: "preview".into(),
                expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
                cutoff_date: cutoff,
                candidates,
            },
        )
        .unwrap();
        let state = take_preview(&token).unwrap();
        assert!(take_preview(&token).is_err());
        let result = execute_cleanup(&dir, &state, today).unwrap();
        assert_eq!(result.removed.len(), 1);
        assert!(!dir.join("admin.2026-06-01").exists());
        assert!(dir.join("admin.2026-08-10").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_refuses_changed_file_between_preview_and_confirm() {
        let dir = temp_log_dir();
        let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let cutoff = today - Days::new(RETENTION_DAYS);
        let path = dir.join("admin.2026-06-01");
        fs::write(&path, b"old").unwrap();
        let (mut candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
        OpenOptions::new().append(true).open(&path).unwrap().write_all(b"changed").unwrap();
        let preview = CleanupPreviewState {
            preview_id: "preview".into(),
            expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
            cutoff_date: cutoff,
            candidates: std::mem::take(&mut candidates),
        };
        let result = execute_cleanup(&dir, &preview, today).unwrap();
        assert!(result.removed.is_empty());
        assert_eq!(result.failures.len(), 1);
        assert!(path.exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_refuses_same_size_replacement_and_symlink_barriers() {
        let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let cutoff = today - Days::new(RETENTION_DAYS);

        let dir = temp_log_dir();
        let path = dir.join("admin.2026-06-01");
        fs::write(&path, b"old").unwrap();
        let (candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
        fs::rename(&path, dir.join("admin.2026-06-01.replaced")).unwrap();
        fs::write(&path, b"new").unwrap();
        let preview = CleanupPreviewState {
            preview_id: "replacement".into(),
            expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
            cutoff_date: cutoff,
            candidates,
        };
        let result = execute_cleanup(&dir, &preview, today).unwrap();
        assert!(result.removed.is_empty());
        assert_eq!(result.failures.len(), 1);
        assert!(path.exists());
        fs::remove_dir_all(&dir).unwrap();

        let dir = temp_log_dir();
        let path = dir.join("admin.2026-06-01");
        fs::write(&path, b"old").unwrap();
        let (candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
        fs::remove_file(&path).unwrap();
        fs::write(dir.join("decoy"), b"decoy").unwrap();
        std::os::unix::fs::symlink("decoy", &path).unwrap();
        let preview = CleanupPreviewState {
            preview_id: "symlink".into(),
            expires_at_instant: Instant::now() + CLEANUP_PREVIEW_TTL,
            cutoff_date: cutoff,
            candidates,
        };
        let result = execute_cleanup(&dir, &preview, today).unwrap();
        assert!(result.removed.is_empty());
        assert_eq!(result.failures.len(), 1);
        assert!(path.is_symlink());
        assert!(dir.join("decoy").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn cleanup_transaction_rejects_same_name_replacement_after_final_check() {
        let dir = temp_log_dir();
        let path = dir.join("admin.2026-06-01");
        let moved_original = dir.join("admin.2026-06-01.original");
        fs::write(&path, b"old").unwrap();
        let today = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let cutoff = today - Days::new(RETENTION_DAYS);
        let (candidates, _) = collect_cleanup_candidates(&dir, today, cutoff).unwrap();
        let candidate = &candidates[0];
        let directory = secure_fs::open_directory(&dir).unwrap().unwrap();
        let result = directory.unlink_if_unchanged_with_barrier(
            &candidate.selector.file_name,
            candidate.signature,
            &candidate.digest,
            || {
                fs::rename(&path, &moved_original).unwrap();
                fs::write(&path, b"new").unwrap();
            },
        );
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"new");
        assert_eq!(fs::read(&moved_original).unwrap(), b"old");
        fs::remove_dir_all(dir).unwrap();
    }

    #[tokio::test]
    async fn audit_records_action_metadata_without_log_content() {
        let pool =
            SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        crate::infra::db::run_migrations(&pool).await.unwrap();
        let user = CurrentUser::new(1, "owner", ["*".to_owned()], true);
        let mut data = operation_data(
            "success",
            json!({
                "files": [{ "module": "admin", "date": "2026-01-01" }]
            }),
            json!({ "fileCount": 1 }),
            None,
        );
        data["archiveSha256"] = json!("abc");
        audit_with_status(
            &pool,
            &user,
            "MODULE_LOG_BACKUP",
            "Backed up one module log file".into(),
            "SUCCESS",
            data,
        )
        .await
        .unwrap();
        let (user_id, username, action, status, data): (i64, String, String, String, Option<String>) =
            sqlx::query_as("SELECT user_id, username, action, status, data FROM operation_logs ORDER BY id DESC LIMIT 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(user_id, 1);
        assert_eq!(username, "owner");
        assert_eq!(action, "MODULE_LOG_BACKUP");
        assert_eq!(status, "SUCCESS");
        let data = data.unwrap();
        assert!(data.contains("archiveSha256"));
        assert!(data.contains("resultCounts"));
        assert!(data.contains("admin"));
        assert!(!data.contains("log line"));
    }

    #[tokio::test]
    async fn cleanup_intent_and_result_are_durable_and_content_free() {
        let pool =
            SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        crate::infra::db::run_migrations(&pool).await.unwrap();
        let user = CurrentUser::new(7, "owner", ["*".to_owned()], true);
        let selection = json!({
            "previewId": "preview-1",
            "files": [{ "module": "admin", "fileName": "admin.2026-01-01", "sizeBytes": 12 }]
        });
        let intent_id = audit_with_status(
            &pool,
            &user,
            "MODULE_LOG_CLEANUP_CONFIRM",
            "Persisted module log cleanup intent".into(),
            "PENDING",
            operation_data(
                "intent",
                selection.clone(),
                json!({ "removedCount": 0, "retainedCount": 0, "failureCount": 0 }),
                None,
            ),
        )
        .await
        .unwrap();
        let result_id = audit_with_status(
            &pool,
            &user,
            "MODULE_LOG_CLEANUP_CONFIRM",
            "Persisted module log cleanup result".into(),
            "PARTIAL",
            operation_data(
                "result",
                json!({ "intentId": intent_id, "selection": selection }),
                json!({ "removedCount": 0, "retainedCount": 1, "failureCount": 1 }),
                Some("item_failed"),
            ),
        )
        .await
        .unwrap();
        let rows: Vec<(i64, String, String, Option<String>)> = sqlx::query_as(
            "SELECT user_id, status, action, data FROM operation_logs WHERE id IN (?, ?) ORDER BY id",
        )
        .bind(intent_id)
        .bind(result_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].0, 7);
        assert_eq!(rows[0].1, "PENDING");
        assert_eq!(rows[1].1, "PARTIAL");
        assert!(rows.iter().all(|row| row.2 == "MODULE_LOG_CLEANUP_CONFIRM"));
        assert!(rows.iter().all(|row| !row.3.as_deref().unwrap().contains("log line")));
    }
}
