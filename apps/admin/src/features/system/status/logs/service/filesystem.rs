use super::*;

pub(super) fn checked_log_root(log_dir: &Path) -> Result<Option<PathBuf>, ServiceError> {
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
pub(super) fn checked_candidate(
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

pub(super) fn read_snapshot(checked: &CheckedCandidate) -> Result<FileSnapshot, ServiceError> {
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
