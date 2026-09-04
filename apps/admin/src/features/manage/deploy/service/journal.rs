use super::*;

pub(super) fn remove_release_file(path: &str) -> Result<(), ServiceError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => {
            tracing::warn!(%error, %path, "Release file cleanup failed; database record retained");
            Err(ServiceError::InvalidOperation(format!("Cannot remove release file: {error}")))
        }
    }
}

pub(super) async fn run_boot_recovery<F, Fut, R, RFut>(
    sentinel: &Path,
    journal: &Path,
    recover: F,
    requeue: R,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>>,
    R: FnOnce() -> RFut,
    RFut: Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    let must_requeue_services = sentinel.exists() || journal.exists();
    write_durable_file(sentinel, b"release recovery is incomplete\n")?;
    recover().await?;
    if must_requeue_services {
        requeue().await?;
    }
    remove_durable_file(sentinel)?;
    Ok(())
}

pub(super) async fn recover_interrupted_update() -> Result<(), Box<dyn std::error::Error>> {
    let Some(journal) = read_update_journal()? else {
        return Ok(());
    };
    tracing::warn!(release_id = journal.release_id, stage = %journal.stage, "Recovering interrupted release update");
    rollback_update(&journal).await?;
    remove_update_journal()?;
    Ok(())
}

pub(super) async fn recover_interrupted_update_without_restart()
-> Result<(), Box<dyn std::error::Error>> {
    let Some(journal) = read_update_journal()? else {
        return Ok(());
    };
    validate_rollback_release(&journal)?;
    let units = journal.restarted_units.iter().map(String::as_str).collect::<Vec<_>>();
    if !units.is_empty() {
        systemctl("stop", &units).await?;
    }
    restore_release_state(&journal, &units)?;
    daemon_reload().await?;
    cleanup_failed_release(&journal)?;
    remove_update_journal()?;
    Ok(())
}

pub(super) fn update_journal_path() -> PathBuf {
    CONFIG.data_dir().join("update-state.json")
}

pub(super) fn read_update_journal() -> Result<Option<UpdateJournal>, Box<dyn std::error::Error>> {
    read_update_journal_at(&update_journal_path())
}

pub(super) fn read_update_journal_at(
    path: &Path,
) -> Result<Option<UpdateJournal>, Box<dyn std::error::Error>> {
    match fs::read(path) {
        Ok(data) => Ok(Some(serde_json::from_slice(&data)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn write_update_journal(
    journal: &UpdateJournal,
) -> Result<(), Box<dyn std::error::Error>> {
    write_update_journal_at(&update_journal_path(), journal)
}

pub(super) fn write_update_journal_at(
    path: &Path,
    journal: &UpdateJournal,
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("invalid update state path"))?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("json.new");
    let mut file =
        fs::OpenOptions::new().write(true).create(true).truncate(true).open(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(journal)?)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, path)?;
    sync_directory(parent)?;
    Ok(())
}

pub(super) fn remove_update_journal() -> Result<(), std::io::Error> {
    let path = update_journal_path();
    match fs::remove_file(&path) {
        Ok(()) => {
            if let Some(parent) = path.parent() {
                sync_directory(parent)?;
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn write_durable_file(path: &Path, data: &[u8]) -> Result<(), std::io::Error> {
    let parent = path.parent().ok_or_else(|| std::io::Error::other("invalid durable file path"))?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("new");
    let mut file =
        fs::OpenOptions::new().write(true).create(true).truncate(true).open(&temporary)?;
    file.write_all(data)?;
    file.sync_all()?;
    drop(file);
    fs::rename(temporary, path)?;
    sync_directory(parent)
}

pub(super) fn remove_durable_file(path: &Path) -> Result<(), std::io::Error> {
    match fs::remove_file(path) {
        Ok(()) => {
            if let Some(parent) = path.parent() {
                sync_directory(parent)?;
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}
