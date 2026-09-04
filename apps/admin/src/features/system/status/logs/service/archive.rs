use super::*;

pub(super) fn build_archive(
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

pub(super) fn manifest_bytes(snapshots: &[FileSnapshot]) -> Result<Vec<u8>, ServiceError> {
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

pub(super) fn append_tar_entry(
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
