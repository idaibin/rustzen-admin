use super::*;

pub(super) async fn backup_databases(version: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let backup_dir = CONFIG.data_dir().join("backups").join(format!(
        "{}-{}",
        version,
        chrono::Utc::now().timestamp()
    ));
    create_dir_all_durable(&backup_dir)?;
    let mut files = BTreeMap::new();
    for path in database_paths() {
        if !path.is_file() {
            return Err(std::io::Error::other(format!(
                "required release database is missing: {}",
                path.display()
            ))
            .into());
        }
        let file_name = path.file_name().ok_or_else(|| std::io::Error::other("invalid db path"))?;
        let destination = backup_dir.join(file_name);
        backup_database_online(&path, &destination).await?;
        files.insert(file_name.to_string_lossy().to_string(), sha256_hex(&fs::read(&destination)?));
    }
    write_backup_manifest(&backup_dir, &BackupManifest { files })?;
    Ok(backup_dir)
}

pub(super) async fn backup_database_online(
    source: &Path,
    destination: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let pool = crate::infra::db::create_pool_for_path(source).await?;
    sqlx::query("VACUUM INTO ?")
        .bind(destination.to_string_lossy().as_ref())
        .execute(&pool)
        .await?;
    pool.close().await;
    fs::OpenOptions::new().read(true).open(destination)?.sync_all()?;
    Ok(())
}

#[cfg(test)]
pub(super) fn backup_database_paths(
    paths: &[PathBuf],
    backup_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = BTreeMap::new();
    for path in paths {
        let file_name = path.file_name().ok_or_else(|| std::io::Error::other("invalid db path"))?;
        let destination = backup_dir.join(file_name);
        fs::copy(path, &destination)?;
        files.insert(file_name.to_string_lossy().to_string(), sha256_hex(&fs::read(destination)?));
    }
    write_backup_manifest(backup_dir, &BackupManifest { files })?;
    Ok(())
}

pub(super) fn write_backup_manifest(
    backup_dir: &Path,
    manifest: &BackupManifest,
) -> Result<(), Box<dyn std::error::Error>> {
    let temporary = backup_dir.join("manifest.json.new");
    fs::write(&temporary, serde_json::to_vec_pretty(manifest)?)?;
    fs::OpenOptions::new().read(true).open(&temporary)?.sync_all()?;
    fs::rename(temporary, backup_dir.join("manifest.json"))?;
    sync_directory(backup_dir)?;
    Ok(())
}

pub(super) fn restore_databases_for_units_at(
    backup_dir: &Path,
    units: &[&str],
    all_paths: &[PathBuf; 4],
) -> Result<(), Box<dyn std::error::Error>> {
    let paths = SYSTEMD_UNITS
        .iter()
        .zip(all_paths.iter())
        .filter_map(|(unit, path)| units.contains(unit).then_some(path.clone()))
        .collect::<Vec<_>>();
    restore_database_paths(&paths, backup_dir)
}

pub(super) fn restore_database_paths(
    paths: &[PathBuf],
    backup_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let manifest: BackupManifest =
        serde_json::from_slice(&fs::read(backup_dir.join("manifest.json"))?)?;
    let mut sources = Vec::new();
    for path in paths {
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| std::io::Error::other("invalid db path"))?;
        let source = backup_dir.join(file_name);
        let expected_hash = manifest.files.get(file_name).ok_or_else(|| {
            std::io::Error::other(format!("backup manifest is missing {file_name}"))
        })?;
        if !source.is_file() || sha256_hex(&fs::read(&source)?) != *expected_hash {
            return Err(std::io::Error::other(format!(
                "database backup is missing or corrupt: {}",
                source.display()
            ))
            .into());
        }
        let parent = path
            .parent()
            .filter(|parent| parent.is_dir())
            .ok_or_else(|| std::io::Error::other("database destination directory is missing"))?;
        let destination_metadata = fs::symlink_metadata(path)?;
        if !destination_metadata.file_type().is_file() {
            return Err(std::io::Error::other("database destination is not a regular file").into());
        }
        sources.push((source, path.clone(), parent.to_path_buf(), destination_metadata));
    }

    let restore_id = uuid::Uuid::new_v4();
    let mut staged = Vec::new();
    for (source, destination, parent, metadata) in &sources {
        let file_name =
            destination.file_name().ok_or_else(|| std::io::Error::other("invalid db path"))?;
        let temporary =
            parent.join(format!(".{}.restore-{restore_id}", file_name.to_string_lossy()));
        fs::copy(source, &temporary)?;
        preserve_database_permissions(&temporary, metadata)?;
        fs::OpenOptions::new().read(true).open(&temporary)?.sync_all()?;
        staged.push((temporary, destination.clone()));
    }
    for (temporary, destination) in &staged {
        fs::rename(temporary, destination)?;
    }
    for (_, destination, _, _) in &sources {
        for suffix in ["-wal", "-shm"] {
            let sidecar = PathBuf::from(format!("{}{suffix}", destination.display()));
            if sidecar.exists() {
                fs::remove_file(sidecar)?;
            }
        }
    }
    for (_, _, parent, _) in &sources {
        sync_directory(parent)?;
    }
    Ok(())
}

#[cfg(unix)]
fn preserve_database_permissions(
    path: &Path,
    original: &fs::Metadata,
) -> Result<(), std::io::Error> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let copied = fs::symlink_metadata(path)?;
    if copied.uid() != original.uid() || copied.gid() != original.gid() {
        std::os::unix::fs::chown(path, Some(original.uid()), Some(original.gid()))?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(original.permissions().mode() & 0o777))
}

#[cfg(not(unix))]
fn preserve_database_permissions(
    path: &Path,
    original: &fs::Metadata,
) -> Result<(), std::io::Error> {
    fs::set_permissions(path, original.permissions())
}

pub(super) fn database_paths() -> [PathBuf; 4] {
    [
        CONFIG.monitor_database_path(),
        CONFIG.insights_database_path(),
        CONFIG.reports_database_path(),
        CONFIG.admin_database_path(),
    ]
}

#[cfg(unix)]
pub(super) fn swap_symlink(link: &Path, target: &Path) -> Result<(), std::io::Error> {
    use std::os::unix::fs::symlink;
    let temporary = link.with_extension("new");
    if fs::symlink_metadata(&temporary).is_ok() {
        fs::remove_file(&temporary)?;
    }
    symlink(target, &temporary)?;
    fs::rename(temporary, link)?;
    sync_directory(link.parent().ok_or_else(|| std::io::Error::other("invalid current link path"))?)
}

pub(super) fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    fs::File::open(path)?.sync_all()
}

pub(super) fn create_dir_all_durable(path: &Path) -> Result<(), std::io::Error> {
    let mut missing = Vec::new();
    let mut current = path;
    while !current.exists() {
        missing.push(current.to_path_buf());
        current = current
            .parent()
            .ok_or_else(|| std::io::Error::other("directory path has no existing ancestor"))?;
    }
    fs::create_dir_all(path)?;
    for directory in missing.iter().rev() {
        sync_directory(directory)?;
        if let Some(parent) = directory.parent() {
            sync_directory(parent)?;
        }
    }
    Ok(())
}
