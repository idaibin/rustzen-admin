use super::*;

pub(super) fn validate_upload_size(data: &[u8]) -> Result<(), ServiceError> {
    if data.is_empty() || data.len() > DEPLOY_FILE_MAX_SIZE {
        return Err(ServiceError::InvalidOperation(format!(
            "Release must be between 1 byte and {DEPLOY_FILE_MAX_SIZE} bytes"
        )));
    }
    Ok(())
}

pub(super) fn validate_version(value: String) -> Result<String, ServiceError> {
    let value = value.trim();
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.len() > 64
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-_".contains(character))
    {
        return Err(ServiceError::InvalidOperation("Invalid release version".to_string()));
    }
    Ok(value.to_string())
}

pub(super) fn normalize_optional(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

pub(super) fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub(super) async fn save_release(
    version: &str,
    arch: &str,
    data: &[u8],
) -> Result<PathBuf, ServiceError> {
    let path = CONFIG.data_dir().join("releases").join(format!("rz-{version}-{arch}.tar"));
    if path.exists() {
        return Err(ServiceError::InvalidOperation("Release file already exists".to_string()));
    }
    fs::create_dir_all(
        path.parent().ok_or_else(|| {
            ServiceError::InvalidOperation("Release path has no parent".to_string())
        })?,
    )
    .map_err(|error| {
        ServiceError::InvalidOperation(format!("Cannot create release upload directory: {error}"))
    })?;
    tokio::fs::write(&path, data)
        .await
        .map_err(|error| ServiceError::InvalidOperation(format!("Cannot save release: {error}")))?;
    fs::OpenOptions::new()
        .read(true)
        .open(&path)
        .and_then(|file| file.sync_all())
        .and_then(|()| {
            path.parent()
                .ok_or_else(|| std::io::Error::other("invalid release path"))
                .and_then(sync_directory)
        })
        .map_err(|error| {
            ServiceError::InvalidOperation(format!("Cannot persist release bundle: {error}"))
        })?;
    Ok(path)
}

pub(super) fn validate_stored_release(
    version: &DeploymentItem,
) -> Result<BundleInfo, ServiceError> {
    let data = fs::read(&version.file_path)
        .map_err(|error| ServiceError::InvalidOperation(format!("Cannot read release: {error}")))?;
    if sha256_hex(&data) != version.file_hash {
        return Err(ServiceError::InvalidOperation("Stored release hash mismatch".to_string()));
    }
    let bundle = validate_bundle(
        &data,
        &version.version,
        CONFIG.deploy_signature_required,
        CONFIG.deploy_verify_key.as_deref(),
    )?;
    if bundle.arch != version.arch {
        return Err(ServiceError::InvalidOperation(
            "Stored release bundle architecture mismatch".to_string(),
        ));
    }
    Ok(bundle)
}

pub(super) fn ensure_version_is_deployable(version: &DeploymentItem) -> Result<(), ServiceError> {
    if version.is_expired || version.deleted_at.is_some() {
        return Err(ServiceError::InvalidOperation(
            "Expired or deleted releases cannot be applied".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_current_link(
    runtime_root: &Path,
    link: &Path,
) -> Result<PathBuf, std::io::Error> {
    let target = fs::read_link(link).map_err(|error| {
        std::io::Error::other(format!(
            "current release link is unavailable at {}: {error}",
            link.display()
        ))
    })?;
    validate_release_target(runtime_root, &target)?;
    Ok(target)
}

pub(super) fn validate_release_target(
    runtime_root: &Path,
    target: &Path,
) -> Result<(), std::io::Error> {
    let components = target.components().collect::<Vec<_>>();
    if target.is_absolute()
        || components.len() != 2
        || components[0] != std::path::Component::Normal("releases".as_ref())
        || !matches!(components[1], std::path::Component::Normal(_))
        || !runtime_root.join(target).is_dir()
    {
        return Err(std::io::Error::other(
            "current release link must target an installed releases/<version> directory",
        ));
    }
    Ok(())
}

pub(super) fn validate_rollback_release(
    journal: &UpdateJournal,
) -> Result<(), Box<dyn std::error::Error>> {
    let runtime_root = journal
        .link
        .parent()
        .ok_or_else(|| std::io::Error::other("current release link has no runtime root"))?;
    validate_release_target(runtime_root, &journal.old_target)?;
    let version = release_version_from_target(&journal.old_target)?;
    let release_dir = runtime_root.join(&journal.old_target);
    let arch = installed_release_arch(&release_dir)?;
    let (data, bundle) = load_installed_bundle(runtime_root, version, arch)?;
    verify_installed_bundle(&data, &bundle, version, &release_dir)?;
    Ok(())
}
