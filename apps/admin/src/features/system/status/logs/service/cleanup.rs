use super::*;

pub(super) fn collect_cleanup_candidates(
    log_dir: &Path,
    today: NaiveDate,
    cutoff_date: NaiveDate,
) -> Result<(Vec<CleanupCandidateState>, Vec<ModuleLogItemFailure>), ServiceError> {
    let mut candidates = Vec::new();
    let mut failures = Vec::new();
    for expected_module in MODULE_IDS {
        let Some(root) = checked_module_log_root(log_dir, expected_module)? else {
            continue;
        };
        let Some(directory) = open_module_log_directory(log_dir, expected_module)? else {
            continue;
        };
        for entry in fs::read_dir(&root).map_err(io_error("read log directory"))? {
            let entry = entry.map_err(io_error("read log directory entry"))?;
            let Some(file_name) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let Some((module, date)) = parse_file_name(&file_name) else {
                continue;
            };
            if module != expected_module || date >= cutoff_date || date >= today {
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
            let modified_at =
                signature.modified.map(DateTime::<Utc>::from).unwrap_or_else(Utc::now);
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
    }
    candidates.sort_by(|left, right| left.candidate.file_name.cmp(&right.candidate.file_name));
    Ok((candidates, failures))
}

pub(super) fn execute_cleanup(
    log_dir: &Path,
    preview: &CleanupPreviewState,
    today: NaiveDate,
) -> Result<ModuleLogCleanupResultResp, ServiceError> {
    let mut removed = Vec::new();
    let mut retained = Vec::new();
    let mut failures = Vec::new();
    for state in &preview.candidates {
        let candidate = state.candidate.clone();
        if state.selector.date >= today || state.selector.date >= preview.cutoff_date {
            retained.push(candidate);
            continue;
        }
        let result =
            open_module_log_directory(log_dir, &state.selector.module).and_then(|directory| {
                directory
                    .ok_or_else(|| {
                        ServiceError::InvalidOperation("Module log directory is unavailable".into())
                    })?
                    .unlink_if_unchanged(&state.selector.file_name, state.signature, &state.digest)
            });
        match result {
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
pub(super) fn store_preview(token: String, state: CleanupPreviewState) -> Result<(), ServiceError> {
    let mut previews = CLEANUP_PREVIEWS.lock().map_err(|_| {
        ServiceError::InvalidOperation("Cleanup confirmation is unavailable".into())
    })?;
    let now = Instant::now();
    previews.retain(|_, preview| preview.expires_at_instant > now);
    previews.insert(token, state);
    Ok(())
}

pub(super) fn remove_preview(token: &str) {
    if let Ok(mut previews) = CLEANUP_PREVIEWS.lock() {
        previews.remove(token);
    }
}

pub(super) fn take_preview(token: &str) -> Result<CleanupPreviewState, ServiceError> {
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
