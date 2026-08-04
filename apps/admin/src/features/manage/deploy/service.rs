use std::{
    collections::BTreeMap,
    fs,
    future::Future,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use axum::{extract::Multipart, http::StatusCode};
use rustzen_ipc::HealthResponse;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::process::Command;

use crate::{
    common::{
        error::ServiceError,
        pagination::{Pagination, PaginationQuery},
    },
    features::manage::deploy::types::{
        DeployComponent, DeploymentItem, DeploymentPayload, ExpireVersionRequest,
        ListDeploymentsQuery,
    },
    infra::config::CONFIG,
};

use super::{
    bundle::{
        BundleInfo, install_bundle, installed_release_arch, validate_bundle,
        verify_installed_bundle,
    },
    repo::DeployRepository,
};

pub const DEPLOY_FILE_MAX_SIZE: usize = 256 * 1024 * 1024;
const DEPLOY_BODY_LIMIT: usize = DEPLOY_FILE_MAX_SIZE + 1024 * 1024;
const SYSTEMD_UNITS: &[&str] =
    &["rz-monitor.service", "rz-insights.service", "rz-reports.service", "rz-admin.service"];

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateJournal {
    release_id: i64,
    backup_dir: PathBuf,
    link: PathBuf,
    old_target: PathBuf,
    new_release_dir: PathBuf,
    install_staging_dir: PathBuf,
    installed_by_update: bool,
    stage: String,
    #[serde(default)]
    restarted_units: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct BackupManifest {
    files: BTreeMap<String, String>,
}

#[derive(Clone)]
pub struct DeployService {
    repo: Arc<DeployRepository>,
}

impl DeployService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self { repo: Arc::new(DeployRepository::new(pool)) }
    }

    pub fn upload_body_limit() -> usize {
        DEPLOY_BODY_LIMIT
    }

    pub async fn bootstrap_installed_current(&self) -> Result<(), ServiceError> {
        let production = matches!(
            CONFIG.runtime.environment.trim().to_ascii_lowercase().as_str(),
            "production" | "prod"
        );
        self.bootstrap_installed_current_at(&CONFIG.runtime_root_dir(), production).await
    }

    async fn bootstrap_installed_current_at(
        &self,
        runtime_root: &Path,
        required: bool,
    ) -> Result<(), ServiceError> {
        let link = runtime_root.join("current");
        if fs::symlink_metadata(&link)
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        {
            return if required {
                Err(ServiceError::InvalidOperation(
                    "Production Admin requires an installed current release".to_string(),
                ))
            } else {
                Ok(())
            };
        }
        let runtime_root = fs::canonicalize(runtime_root).map_err(|error| {
            ServiceError::InvalidOperation(format!("Cannot resolve runtime root: {error}"))
        })?;
        let target = validate_current_link(&runtime_root, &runtime_root.join("current"))
            .map_err(|error| ServiceError::InvalidOperation(error.to_string()))?;
        let version = release_version_from_target(&target)
            .map_err(|error| ServiceError::InvalidOperation(error.to_string()))?
            .to_string();
        let release_dir = runtime_root.join(&target);
        let arch = installed_release_arch(&release_dir)?.to_string();
        let (data, bundle) = load_installed_bundle(&runtime_root, &version, &arch)
            .map_err(|error| ServiceError::InvalidOperation(error.to_string()))?;
        verify_installed_bundle(&data, &bundle, &version, &release_dir)?;
        let bundle_path =
            runtime_root.join("data/releases").join(format!("rz-{version}-{arch}.tar"));
        if runtime_root.join("data/update-state.json").is_file() || self.repo.has_current().await? {
            return Ok(());
        }
        self.repo
            .upsert_installed_current(&DeploymentPayload {
                component: DeployComponent::Release,
                version,
                arch,
                file_path: bundle_path.to_string_lossy().to_string(),
                file_size: i64::try_from(data.len()).unwrap_or(i64::MAX),
                file_hash: sha256_hex(&data),
                notes: Some("Registered from installed current release".to_string()),
            })
            .await
    }

    pub async fn list(
        &self,
        query: ListDeploymentsQuery,
    ) -> Result<(Vec<DeploymentItem>, i64), ServiceError> {
        let pagination = Pagination::from_query(PaginationQuery {
            current: query.current,
            page_size: query.page_size,
        });
        self.repo.list(&query, pagination.offset.into(), pagination.limit.into()).await
    }

    pub async fn upload(&self, mut multipart: Multipart) -> Result<DeploymentItem, ServiceError> {
        let mut version = None;
        let mut arch = None;
        let mut notes = None;
        let mut file_data = None;

        while let Some(field) = multipart.next_field().await.map_err(|error| {
            if error.status() == StatusCode::PAYLOAD_TOO_LARGE
                || error.body_text() == "Request payload is too large"
            {
                ServiceError::PayloadTooLarge
            } else {
                ServiceError::InvalidOperation("Invalid multipart data".to_string())
            }
        })? {
            let Some(name) = field.name().map(str::to_string) else {
                continue;
            };
            match name.as_str() {
                "component" => {
                    let value = field.text().await.map_err(|_| {
                        ServiceError::InvalidOperation("Invalid component field".to_string())
                    })?;
                    if value != "release" {
                        return Err(ServiceError::InvalidOperation(
                            "Only complete release artifacts are accepted".to_string(),
                        ));
                    }
                }
                "version" => {
                    version = Some(validate_version(field.text().await.map_err(|_| {
                        ServiceError::InvalidOperation("Invalid version field".to_string())
                    })?)?);
                }
                "arch" => {
                    arch = normalize_optional(field.text().await.map_err(|_| {
                        ServiceError::InvalidOperation("Invalid arch field".to_string())
                    })?);
                }
                "notes" => notes = normalize_optional(field.text().await.unwrap_or_default()),
                "file" => {
                    file_data = Some(
                        field
                            .bytes()
                            .await
                            .map_err(|error| {
                                if error.status() == StatusCode::PAYLOAD_TOO_LARGE
                                    || error.body_text() == "Request payload is too large"
                                {
                                    ServiceError::PayloadTooLarge
                                } else {
                                    ServiceError::InvalidOperation(
                                        "Failed to read uploaded release".to_string(),
                                    )
                                }
                            })?
                            .to_vec(),
                    );
                }
                _ => {}
            }
        }

        let version =
            version.ok_or_else(|| ServiceError::InvalidOperation("version is required".into()))?;
        let data =
            file_data.ok_or_else(|| ServiceError::InvalidOperation("file is required".into()))?;
        validate_upload_size(&data)?;
        let bundle = validate_bundle(
            &data,
            &version,
            CONFIG.deploy_signature_required,
            CONFIG.deploy_verify_key.as_deref(),
        )?;
        let detected_arch = bundle.arch;
        if let Some(requested_arch) = arch
            && requested_arch != detected_arch
        {
            return Err(ServiceError::InvalidOperation(
                "Uploaded release architecture does not match arch field".to_string(),
            ));
        }
        let component = DeployComponent::Release;
        if self.repo.version_exists(&component, &version, &detected_arch).await? {
            return Err(ServiceError::InvalidOperation(
                "Release version has already been uploaded for this architecture".to_string(),
            ));
        }
        let file_hash = sha256_hex(&data);
        let file_path = save_release(&version, &detected_arch, &data).await?;
        match self
            .repo
            .insert(&DeploymentPayload {
                component,
                version,
                arch: detected_arch,
                file_path: file_path.to_string_lossy().to_string(),
                file_size: i64::try_from(data.len()).unwrap_or(i64::MAX),
                file_hash,
                notes,
            })
            .await
        {
            Ok(item) => Ok(item),
            Err(error) => {
                if let Err(cleanup_error) = fs::remove_file(&file_path) {
                    tracing::error!(
                        %cleanup_error,
                        path = %file_path.display(),
                        "Failed to remove unregistered release artifact"
                    );
                }
                Err(error)
            }
        }
    }

    pub async fn find_by_id(&self, id: i64) -> Result<DeploymentItem, ServiceError> {
        self.repo.find_by_id(id).await
    }

    pub async fn deploy(&self, id: i64, deployed_by: String) -> Result<bool, ServiceError> {
        let version = self.repo.find_by_id(id).await?;
        ensure_version_is_deployable(&version)?;
        validate_stored_release(&version)?;
        let executable = std::env::current_exe().map_err(|error| {
            ServiceError::InvalidOperation(format!("Cannot locate rz-admin executable: {error}"))
        })?;
        let runtime_root = fs::canonicalize(CONFIG.runtime_root_dir()).map_err(|error| {
            ServiceError::InvalidOperation(format!("Cannot resolve runtime root: {error}"))
        })?;
        let environment_file = runtime_root.join("config/rz.env");
        if !environment_file.is_file() {
            return Err(ServiceError::InvalidOperation(format!(
                "Update worker environment file is missing: {}",
                environment_file.display()
            )));
        }
        let status = Command::new("systemd-run")
            .arg("--unit=rz-update")
            .args(["--collect", "--no-block"])
            .args([
                "--property=Type=exec",
                "--property=Restart=on-failure",
                "--property=RestartSec=2s",
                "--property=StartLimitIntervalSec=60s",
                "--property=StartLimitBurst=3",
            ])
            .arg(format!("--property=EnvironmentFile={}", environment_file.display()))
            .arg(format!("--working-directory={}", runtime_root.display()))
            .arg(format!("--setenv=RUSTZEN_UPDATE_DEPLOYED_BY={deployed_by}"))
            .arg(executable)
            .args(["update", "worker", &id.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(|error| {
                ServiceError::InvalidOperation(format!("Failed to start update worker: {error}"))
            })?;
        if !status.success() {
            return Err(ServiceError::InvalidOperation(
                "Failed to enqueue update worker; another update may be active".to_string(),
            ));
        }
        Ok(true)
    }

    pub async fn expire(
        &self,
        id: i64,
        request: ExpireVersionRequest,
    ) -> Result<DeploymentItem, ServiceError> {
        self.repo.mark_expired(id, request.notes.as_deref()).await
    }

    pub async fn delete(&self, id: i64) -> Result<DeploymentItem, ServiceError> {
        let version = self.repo.find_by_id(id).await?;
        if version.is_current {
            return Err(ServiceError::InvalidOperation(
                "Cannot delete the current release".to_string(),
            ));
        }
        let deleted = self.repo.delete_by_id(id).await?;
        if let Err(error) = fs::remove_file(&deleted.file_path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(%error, path = %deleted.file_path, "Release file cleanup failed");
        }
        Ok(deleted)
    }

    pub async fn cleanup_expired(
        &self,
        _component: Option<DeployComponent>,
    ) -> Result<usize, ServiceError> {
        let versions = self.repo.expired_non_current(Some(&DeployComponent::Release)).await?;
        let mut deleted = 0;
        for version in versions {
            let deleted_version = self.repo.delete_by_id(version.id).await?;
            if let Err(error) = fs::remove_file(&deleted_version.file_path)
                && error.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(%error, path = %deleted_version.file_path, "Release cleanup failed");
            }
            deleted += 1;
        }
        Ok(deleted)
    }

    pub async fn run_update_worker(id: i64) -> Result<(), Box<dyn std::error::Error>> {
        recover_interrupted_update().await?;
        let pool = crate::infra::db::create_default_pool().await?;
        crate::infra::db::run_migrations(&pool).await?;
        let service = Self::new(pool.clone());
        let version = service.find_by_id(id).await?;
        ensure_version_is_deployable(&version)?;
        let bundle = validate_stored_release(&version)?;
        drop(service);
        pool.close().await;

        let runtime_root = fs::canonicalize(CONFIG.runtime_root_dir())?;
        let link = runtime_root.join("current");
        let old_target = validate_current_link(&runtime_root, &link)?;
        let old_version = release_version_from_target(&old_target)?.to_string();
        if old_version == version.version {
            return Err(std::io::Error::other("release is already current").into());
        }
        let old_release_dir = runtime_root.join(&old_target);
        let current_arch = installed_release_arch(&old_release_dir)?;
        if bundle.arch != current_arch {
            return Err(std::io::Error::other(format!(
                "release bundle architecture {} does not match installed architecture {current_arch}",
                bundle.arch
            ))
            .into());
        }
        let (current_data, current_bundle) =
            load_installed_bundle(&runtime_root, &old_version, current_arch)?;
        verify_installed_bundle(&current_data, &current_bundle, &old_version, &old_release_dir)?;

        let new_release_dir = runtime_root.join("releases").join(&version.version);
        let data = fs::read(&version.file_path)?;
        let installed_by_update = !new_release_dir.exists();
        if !installed_by_update {
            verify_installed_bundle(&data, &bundle, &version.version, &new_release_dir)?;
        }
        let backup_dir = backup_databases(&version.version).await?;
        let install_staging_dir = runtime_root
            .join("releases")
            .join(format!(".{}.{}.installing", version.version, version.id));
        let mut journal = UpdateJournal {
            release_id: id,
            backup_dir: backup_dir.clone(),
            link: link.clone(),
            old_target: old_target.clone(),
            new_release_dir: new_release_dir.clone(),
            install_staging_dir,
            installed_by_update,
            stage: "backedUp".to_string(),
            restarted_units: Vec::new(),
        };
        write_update_journal(&journal)?;

        let update_result = async {
            journal.stage = "installing".to_string();
            write_update_journal(&journal)?;
            if installed_by_update {
                let installed =
                    install_bundle(&data, &bundle, &version.version, version.id, &runtime_root)?;
                if installed != new_release_dir {
                    return Err(std::io::Error::other(
                        "installed release path changed unexpectedly",
                    )
                    .into());
                }
            }
            journal.stage = "installed".to_string();
            write_update_journal(&journal)?;

            let relative_target = PathBuf::from("releases").join(&version.version);
            swap_symlink(&link, &relative_target)?;
            daemon_reload().await?;
            journal.stage = "switched".to_string();
            write_update_journal(&journal)?;

            roll_services(&mut journal, &update_journal_path(), &version.version).await?;
            journal.stage = "committing".to_string();
            write_update_journal(&journal)?;
            let pool = crate::infra::db::create_default_pool().await?;
            crate::infra::db::run_migrations(&pool).await?;
            let service = Self::new(pool.clone());
            service
                .repo
                .set_current(
                    &DeployComponent::Release,
                    &version.arch,
                    version.id,
                    std::env::var("RUSTZEN_UPDATE_DEPLOYED_BY").ok().as_deref(),
                )
                .await?;
            pool.close().await;
            Ok::<(), Box<dyn std::error::Error>>(())
        }
        .await;

        if let Err(error) = update_result {
            let rollback_result = rollback_update(&journal).await;
            return match rollback_result {
                Ok(()) => {
                    remove_update_journal()?;
                    tracing::error!(%error, "Release update failed and was rolled back");
                    Ok(())
                }
                Err(rollback_error) => Err(std::io::Error::other(format!(
                    "release update failed: {error}; rollback failed: {rollback_error}"
                ))
                .into()),
            };
        }
        remove_update_journal()?;
        Ok(())
    }

    pub async fn recover_interrupted_update_at_boot() -> Result<(), Box<dyn std::error::Error>> {
        let sentinel = CONFIG.data_dir().join("recovery-blocked");
        run_boot_recovery(
            &sentinel,
            &update_journal_path(),
            recover_interrupted_update_without_restart,
            requeue_services_after_recovery,
        )
        .await
    }
}

async fn run_boot_recovery<F, Fut, R, RFut>(
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

async fn recover_interrupted_update() -> Result<(), Box<dyn std::error::Error>> {
    let Some(journal) = read_update_journal()? else {
        return Ok(());
    };
    tracing::warn!(release_id = journal.release_id, stage = %journal.stage, "Recovering interrupted release update");
    rollback_update(&journal).await?;
    remove_update_journal()?;
    Ok(())
}

async fn recover_interrupted_update_without_restart() -> Result<(), Box<dyn std::error::Error>> {
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

fn update_journal_path() -> PathBuf {
    CONFIG.data_dir().join("update-state.json")
}

fn read_update_journal() -> Result<Option<UpdateJournal>, Box<dyn std::error::Error>> {
    read_update_journal_at(&update_journal_path())
}

fn read_update_journal_at(
    path: &Path,
) -> Result<Option<UpdateJournal>, Box<dyn std::error::Error>> {
    match fs::read(path) {
        Ok(data) => Ok(Some(serde_json::from_slice(&data)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn write_update_journal(journal: &UpdateJournal) -> Result<(), Box<dyn std::error::Error>> {
    write_update_journal_at(&update_journal_path(), journal)
}

fn write_update_journal_at(
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

fn remove_update_journal() -> Result<(), std::io::Error> {
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

fn write_durable_file(path: &Path, data: &[u8]) -> Result<(), std::io::Error> {
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

fn remove_durable_file(path: &Path) -> Result<(), std::io::Error> {
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

fn validate_upload_size(data: &[u8]) -> Result<(), ServiceError> {
    if data.is_empty() || data.len() > DEPLOY_FILE_MAX_SIZE {
        return Err(ServiceError::InvalidOperation(format!(
            "Release must be between 1 byte and {DEPLOY_FILE_MAX_SIZE} bytes"
        )));
    }
    Ok(())
}

fn validate_version(value: String) -> Result<String, ServiceError> {
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

fn normalize_optional(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

async fn save_release(version: &str, arch: &str, data: &[u8]) -> Result<PathBuf, ServiceError> {
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

fn validate_stored_release(version: &DeploymentItem) -> Result<BundleInfo, ServiceError> {
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

fn ensure_version_is_deployable(version: &DeploymentItem) -> Result<(), ServiceError> {
    if version.is_expired || version.deleted_at.is_some() {
        return Err(ServiceError::InvalidOperation(
            "Expired or deleted releases cannot be applied".to_string(),
        ));
    }
    Ok(())
}

fn validate_current_link(runtime_root: &Path, link: &Path) -> Result<PathBuf, std::io::Error> {
    let target = fs::read_link(link).map_err(|error| {
        std::io::Error::other(format!(
            "current release link is unavailable at {}: {error}",
            link.display()
        ))
    })?;
    validate_release_target(runtime_root, &target)?;
    Ok(target)
}

fn validate_release_target(runtime_root: &Path, target: &Path) -> Result<(), std::io::Error> {
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

fn validate_rollback_release(journal: &UpdateJournal) -> Result<(), Box<dyn std::error::Error>> {
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

fn load_installed_bundle(
    runtime_root: &Path,
    version: &str,
    arch: &str,
) -> Result<(Vec<u8>, BundleInfo), Box<dyn std::error::Error>> {
    let path = runtime_root.join("data/releases").join(format!("rz-{version}-{arch}.tar"));
    let data = fs::read(&path).map_err(|error| {
        std::io::Error::other(format!(
            "installed release bundle is unavailable at {}: {error}",
            path.display()
        ))
    })?;
    let bundle = validate_bundle(
        &data,
        version,
        CONFIG.deploy_signature_required,
        CONFIG.deploy_verify_key.as_deref(),
    )?;
    let expected_name = format!("rz-{version}-{}.tar", bundle.arch);
    if path.file_name().and_then(|value| value.to_str()) != Some(expected_name.as_str()) {
        return Err(std::io::Error::other("stored installed bundle architecture mismatch").into());
    }
    Ok((data, bundle))
}

fn service_probes() -> [(&'static str, String); 4] {
    [
        ("rz-monitor.service", format!("{}/health", CONFIG.monitor_base_url())),
        ("rz-insights.service", format!("{}/health", CONFIG.insights_base_url())),
        ("rz-reports.service", format!("{}/health", CONFIG.reports_base_url())),
        ("rz-admin.service", format!("http://127.0.0.1:{}/health", CONFIG.admin_port())),
    ]
}

async fn roll_services(
    journal: &mut UpdateJournal,
    journal_path: &Path,
    expected_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder().timeout(Duration::from_secs(2)).build()?;
    roll_services_with(journal, journal_path, |unit, url| {
        let client = client.clone();
        let expected_version = expected_version.to_string();
        async move { restart_and_verify(&unit, &url, &expected_version, &client).await }
    })
    .await
}

async fn roll_services_with<F, Fut>(
    journal: &mut UpdateJournal,
    journal_path: &Path,
    mut restart: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(String, String) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    for (unit, url) in service_probes() {
        journal.stage = format!("restarting:{unit}");
        if !journal.restarted_units.iter().any(|value| value == unit) {
            journal.restarted_units.push(unit.to_string());
        }
        write_update_journal_at(journal_path, journal)?;
        restart(unit.to_string(), url).await?;
    }
    Ok(())
}

async fn restart_and_verify(
    unit: &str,
    url: &str,
    expected_version: &str,
    client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error>> {
    systemctl("stop", &[unit]).await?;
    systemctl("start", &[unit]).await?;
    ensure_unit_active(unit).await?;
    wait_for_health(client, url, expected_version).await
}

async fn rollback_update(journal: &UpdateJournal) -> Result<(), Box<dyn std::error::Error>> {
    validate_rollback_release(journal)?;
    let units = journal.restarted_units.iter().map(String::as_str).collect::<Vec<_>>();
    if !units.is_empty() {
        systemctl("stop", &units).await?;
    }
    restore_release_state(journal, &units)?;
    daemon_reload().await?;
    let old_version = release_version_from_target(&journal.old_target)?;
    let client = reqwest::Client::builder().timeout(Duration::from_secs(2)).build()?;
    for (unit, url) in service_probes() {
        if units.contains(&unit) {
            systemctl("start", &[unit]).await?;
            ensure_unit_active(unit).await?;
            wait_for_health(&client, &url, old_version).await?;
        }
    }
    cleanup_failed_release(journal)?;
    Ok(())
}

fn restore_release_state(
    journal: &UpdateJournal,
    units: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    restore_release_state_with_paths(journal, units, &database_paths())
}

fn restore_release_state_with_paths(
    journal: &UpdateJournal,
    units: &[&str],
    paths: &[PathBuf; 4],
) -> Result<(), Box<dyn std::error::Error>> {
    swap_symlink(&journal.link, &journal.old_target)?;
    restore_databases_for_units_at(&journal.backup_dir, units, paths)
}

fn cleanup_failed_release(journal: &UpdateJournal) -> Result<(), std::io::Error> {
    if journal.installed_by_update
        && journal.stage != "backedUp"
        && journal.new_release_dir.exists()
    {
        fs::remove_dir_all(&journal.new_release_dir)?;
    }
    if journal.install_staging_dir.exists() {
        fs::remove_dir_all(&journal.install_staging_dir)?;
    }
    Ok(())
}

async fn systemctl(action: &str, units: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    if !matches!(action, "start" | "stop") || units.iter().any(|unit| !SYSTEMD_UNITS.contains(unit))
    {
        return Err(std::io::Error::other("invalid fixed service operation").into());
    }
    let status = Command::new("systemctl").arg(action).args(units).status().await?;
    if !status.success() {
        return Err(std::io::Error::other(format!("systemctl {action} failed")).into());
    }
    Ok(())
}

async fn requeue_services_after_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("systemctl")
        .args(["--no-block", "start"])
        .args(SYSTEMD_UNITS)
        .status()
        .await?;
    if !status.success() {
        return Err(std::io::Error::other(
            "systemctl failed to requeue services after release recovery",
        )
        .into());
    }
    Ok(())
}

async fn daemon_reload() -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("systemctl").arg("daemon-reload").status().await?;
    if !status.success() {
        return Err(std::io::Error::other("systemctl daemon-reload failed").into());
    }
    Ok(())
}

async fn ensure_unit_active(unit: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !SYSTEMD_UNITS.contains(&unit) {
        return Err(std::io::Error::other("invalid fixed service operation").into());
    }
    let status = Command::new("systemctl").args(["is-active", "--quiet", unit]).status().await?;
    if !status.success() {
        return Err(std::io::Error::other(format!("systemd unit is not active: {unit}")).into());
    }
    Ok(())
}

async fn wait_for_health(
    client: &reqwest::Client,
    url: &str,
    expected_version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for _ in 0..20 {
        if let Ok(response) = client.get(url).send().await
            && response.status().is_success()
            && let Ok(body) = response.json::<HealthResponse>().await
            && body.status == "ok"
            && body.release_version == expected_version
        {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Err(std::io::Error::other(format!("health gate failed for release {expected_version}: {url}"))
        .into())
}

fn release_version_from_target(target: &Path) -> Result<&str, Box<dyn std::error::Error>> {
    target
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| std::io::Error::other("current release target has no version").into())
}

async fn backup_databases(version: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
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

async fn backup_database_online(
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
fn backup_database_paths(
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

fn write_backup_manifest(
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

fn restore_databases_for_units_at(
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

fn restore_database_paths(
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
        sources.push((source, path.clone(), parent.to_path_buf()));
    }

    let restore_id = uuid::Uuid::new_v4();
    let mut staged = Vec::new();
    for (source, destination, parent) in &sources {
        let file_name =
            destination.file_name().ok_or_else(|| std::io::Error::other("invalid db path"))?;
        let temporary =
            parent.join(format!(".{}.restore-{restore_id}", file_name.to_string_lossy()));
        fs::copy(source, &temporary)?;
        fs::OpenOptions::new().read(true).open(&temporary)?.sync_all()?;
        staged.push((temporary, destination.clone()));
    }
    for (temporary, destination) in &staged {
        fs::rename(temporary, destination)?;
    }
    for (_, destination, _) in &sources {
        for suffix in ["-wal", "-shm"] {
            let sidecar = PathBuf::from(format!("{}{suffix}", destination.display()));
            if sidecar.exists() {
                fs::remove_file(sidecar)?;
            }
        }
    }
    for (_, _, parent) in &sources {
        sync_directory(parent)?;
    }
    Ok(())
}

fn database_paths() -> [PathBuf; 4] {
    [
        CONFIG.monitor_database_path(),
        CONFIG.insights_database_path(),
        CONFIG.reports_database_path(),
        CONFIG.admin_database_path(),
    ]
}

#[cfg(unix)]
fn swap_symlink(link: &Path, target: &Path) -> Result<(), std::io::Error> {
    use std::os::unix::fs::symlink;
    let temporary = link.with_extension("new");
    if fs::symlink_metadata(&temporary).is_ok() {
        fs::remove_file(&temporary)?;
    }
    symlink(target, &temporary)?;
    fs::rename(temporary, link)?;
    sync_directory(link.parent().ok_or_else(|| std::io::Error::other("invalid current link path"))?)
}

fn sync_directory(path: &Path) -> Result<(), std::io::Error> {
    fs::File::open(path)?.sync_all()
}

fn create_dir_all_durable(path: &Path) -> Result<(), std::io::Error> {
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

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    use super::{
        SYSTEMD_UNITS, UpdateJournal, backup_database_online, backup_database_paths,
        cleanup_failed_release, create_dir_all_durable, load_installed_bundle,
        read_update_journal_at, restore_database_paths, restore_release_state_with_paths,
        roll_services_with, run_boot_recovery, swap_symlink, validate_upload_size,
        validate_version, write_update_journal_at,
    };

    #[tokio::test]
    async fn boot_recovery_requeues_services_after_a_failed_first_attempt() {
        let root = std::env::temp_dir().join(format!("rz-boot-recovery-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("recovery root");
        let sentinel = root.join("recovery-blocked");
        let journal = root.join("update-state.json");
        fs::write(&journal, b"pending").expect("journal marker");

        let first = run_boot_recovery(
            &sentinel,
            &journal,
            || async { Err(std::io::Error::other("injected recovery failure").into()) },
            || async { panic!("failed recovery must not requeue services") },
        )
        .await;
        assert!(first.is_err());
        assert!(sentinel.is_file());

        let requeued = Arc::new(Mutex::new(false));
        let observed = Arc::clone(&requeued);
        let sentinel_during_requeue = sentinel.clone();
        run_boot_recovery(
            &sentinel,
            &journal,
            || async { Ok(()) },
            || async move {
                assert!(sentinel_during_requeue.is_file());
                *observed.lock().expect("requeue state") = true;
                Ok(())
            },
        )
        .await
        .expect("second recovery attempt");
        assert!(*requeued.lock().expect("requeue result"));
        assert!(!sentinel.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[tokio::test]
    async fn boot_recovery_keeps_services_blocked_when_requeue_fails() {
        let root = std::env::temp_dir().join(format!("rz-boot-requeue-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("recovery root");
        let sentinel = root.join("recovery-blocked");
        let journal = root.join("update-state.json");
        fs::write(&journal, b"pending").expect("journal marker");

        let result = run_boot_recovery(
            &sentinel,
            &journal,
            || async { Ok(()) },
            || async { Err(std::io::Error::other("injected requeue failure").into()) },
        )
        .await;
        assert!(result.is_err());
        assert!(sentinel.is_file());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn durable_directory_creation_persists_the_full_parent_chain() {
        let root = std::env::temp_dir().join(format!("rz-durable-dir-{}", uuid::Uuid::new_v4()));
        let leaf = root.join("data/backups/release");
        create_dir_all_durable(&leaf).expect("durable directory chain");
        assert!(leaf.is_dir());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn validates_release_upload_size_and_version() {
        assert!(validate_upload_size(b"bundle").is_ok());
        assert_eq!(validate_version("1.2.3".to_string()).expect("version"), "1.2.3");
    }

    #[test]
    fn rejects_invalid_release_inputs() {
        assert!(validate_upload_size(&[]).is_err());
        assert!(validate_version("../module".to_string()).is_err());
        assert!(validate_version(".".to_string()).is_err());
        assert!(validate_version("..".to_string()).is_err());
    }

    #[test]
    fn installed_bundle_selection_uses_the_running_architecture() {
        let root = std::env::temp_dir().join(format!("rz-installed-arch-{}", uuid::Uuid::new_v4()));
        let releases = root.join("data/releases");
        fs::create_dir_all(&releases).expect("release uploads");
        for arch in ["x86_64", "aarch64"] {
            fs::write(
                releases.join(format!("rz-1.2.3-{arch}.tar")),
                crate::features::manage::deploy::bundle::tests::fixture("1.2.3", arch),
            )
            .expect("bundle");
        }
        let (_, selected) =
            load_installed_bundle(&root, "1.2.3", "x86_64").expect("selected bundle");
        assert_eq!(selected.arch, "x86_64");
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn fresh_install_bootstraps_exactly_one_current_release_row() {
        let version = "1.2.3";
        let arch = "x86_64";
        let root =
            std::env::temp_dir().join(format!("rz-bootstrap-current-{}", uuid::Uuid::new_v4()));
        let data = crate::features::manage::deploy::bundle::tests::fixture(version, arch);
        let info =
            crate::features::manage::deploy::bundle::validate_bundle(&data, version, false, None)
                .expect("bundle");
        crate::features::manage::deploy::bundle::install_bundle(&data, &info, version, 1, &root)
            .expect("installed release");
        fs::create_dir_all(root.join("data/releases")).expect("release store");
        fs::write(root.join(format!("data/releases/rz-{version}-{arch}.tar")), &data)
            .expect("stored bundle");
        std::os::unix::fs::symlink(format!("releases/{version}"), root.join("current"))
            .expect("current link");

        let pool = crate::infra::db::create_pool_for_path(&root.join("admin-test.db"))
            .await
            .expect("test pool");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");
        let service = super::DeployService::new(pool.clone());
        service.bootstrap_installed_current_at(&root, true).await.expect("bootstrap current");
        service.bootstrap_installed_current_at(&root, true).await.expect("idempotent bootstrap");
        let current: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM deploy_versions WHERE is_current = 1 AND version = ? AND arch = ?",
        )
        .bind(version)
        .bind(arch)
        .fetch_one(&pool)
        .await
        .expect("current count");
        assert_eq!(current, 1);

        let candidate_version = "2.0.0";
        let candidate =
            crate::features::manage::deploy::bundle::tests::fixture(candidate_version, arch);
        let candidate_info = crate::features::manage::deploy::bundle::validate_bundle(
            &candidate,
            candidate_version,
            false,
            None,
        )
        .expect("candidate bundle");
        crate::features::manage::deploy::bundle::install_bundle(
            &candidate,
            &candidate_info,
            candidate_version,
            2,
            &root,
        )
        .expect("candidate install");
        fs::write(root.join(format!("data/releases/rz-{candidate_version}-{arch}.tar")), candidate)
            .expect("candidate stored bundle");
        swap_symlink(&root.join("current"), Path::new("releases/2.0.0"))
            .expect("candidate current link");
        fs::write(root.join("data/update-state.json"), b"rollout in progress")
            .expect("update journal sentinel");
        service
            .bootstrap_installed_current_at(&root, true)
            .await
            .expect("rollout startup validation");
        let still_current: String =
            sqlx::query_scalar("SELECT version FROM deploy_versions WHERE is_current = 1")
                .fetch_one(&pool)
                .await
                .expect("current version");
        assert_eq!(still_current, version);
        pool.close().await;
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn update_journal_round_trips_interrupted_state() {
        let root = std::env::temp_dir().join(format!("rz-update-{}", uuid::Uuid::new_v4()));
        let path = root.join("data/update-state.json");
        let journal = UpdateJournal {
            release_id: 7,
            backup_dir: root.join("backup"),
            link: root.join("current"),
            old_target: PathBuf::from("releases/old"),
            new_release_dir: root.join("releases/new"),
            install_staging_dir: root.join("releases/.new.7.installing"),
            installed_by_update: true,
            stage: "switched".to_string(),
            restarted_units: Vec::new(),
        };

        write_update_journal_at(&path, &journal).expect("write journal");
        let restored = read_update_journal_at(&path).expect("read journal").expect("journal");
        assert_eq!(restored.release_id, journal.release_id);
        assert_eq!(restored.backup_dir, journal.backup_dir);
        assert_eq!(restored.old_target, journal.old_target);
        assert_eq!(restored.stage, "switched");
        assert!(restored.restarted_units.is_empty());
        fs::remove_dir_all(root).expect("remove journal root");
    }

    #[tokio::test]
    async fn every_service_health_failure_stops_the_roll_sequence() {
        for (failed_index, failed_unit) in SYSTEMD_UNITS.iter().enumerate() {
            let root =
                std::env::temp_dir().join(format!("rz-health-gate-{}", uuid::Uuid::new_v4()));
            let journal_path = root.join("update-state.json");
            let mut journal = UpdateJournal {
                release_id: 9,
                backup_dir: root.join("backup"),
                link: root.join("current"),
                old_target: PathBuf::from("releases/old"),
                new_release_dir: root.join("releases/new"),
                install_staging_dir: root.join("releases/.new.9.installing"),
                installed_by_update: true,
                stage: "switched".to_string(),
                restarted_units: Vec::new(),
            };
            let attempts = Arc::new(Mutex::new(Vec::new()));
            let recorded_attempts = Arc::clone(&attempts);
            let failed_unit = failed_unit.to_string();

            let result = roll_services_with(&mut journal, &journal_path, move |unit, _url| {
                let attempts = Arc::clone(&recorded_attempts);
                let failed_unit = failed_unit.clone();
                async move {
                    attempts.lock().expect("attempt lock").push(unit.clone());
                    if unit == failed_unit {
                        Err(std::io::Error::other("injected health failure").into())
                    } else {
                        Ok(())
                    }
                }
            })
            .await;

            assert!(result.is_err());
            assert_eq!(
                *attempts.lock().expect("attempt lock"),
                SYSTEMD_UNITS[..=failed_index]
                    .iter()
                    .map(|unit| unit.to_string())
                    .collect::<Vec<_>>()
            );
            assert_eq!(journal.restarted_units.len(), failed_index + 1);
            let saved = read_update_journal_at(&journal_path)
                .expect("read journal")
                .expect("saved journal");
            assert_eq!(saved.restarted_units, journal.restarted_units);
            fs::remove_dir_all(root).expect("remove health gate root");
        }
    }

    #[cfg(unix)]
    #[test]
    fn apply_and_each_health_gate_rollback_restore_one_release_boundary() {
        for failed_index in 0..SYSTEMD_UNITS.len() {
            let root = std::env::temp_dir()
                .join(format!("rz-apply-rollback-{failed_index}-{}", uuid::Uuid::new_v4()));
            let releases = root.join("releases");
            let data = root.join("data");
            let backup = root.join("backup");
            fs::create_dir_all(releases.join("old")).expect("old release");
            fs::create_dir_all(releases.join("new")).expect("new release");
            fs::create_dir_all(&data).expect("data");
            fs::create_dir_all(&backup).expect("backup");
            let paths = ["monitor", "insights", "reports", "admin"]
                .map(|name| data.join(format!("{name}.db")));
            for path in &paths {
                fs::write(path, "old-database").expect("old database");
            }
            backup_database_paths(&paths, &backup).expect("backup");
            for path in &paths {
                fs::write(path, "new-database").expect("new database");
            }

            let current = root.join("current");
            std::os::unix::fs::symlink("releases/old", &current).expect("initial current");
            swap_symlink(&current, Path::new("releases/new")).expect("apply release");
            assert_eq!(
                fs::read_link(&current).expect("applied current"),
                PathBuf::from("releases/new")
            );

            let staging = releases.join(format!(".new.{failed_index}.installing"));
            fs::create_dir(&staging).expect("staging");
            let journal = UpdateJournal {
                release_id: failed_index as i64 + 1,
                backup_dir: backup,
                link: current.clone(),
                old_target: PathBuf::from("releases/old"),
                new_release_dir: releases.join("new"),
                install_staging_dir: staging,
                installed_by_update: true,
                stage: format!("restarting:{}", SYSTEMD_UNITS[failed_index]),
                restarted_units: SYSTEMD_UNITS[..=failed_index]
                    .iter()
                    .map(|unit| unit.to_string())
                    .collect(),
            };
            let units = journal.restarted_units.iter().map(String::as_str).collect::<Vec<_>>();
            restore_release_state_with_paths(&journal, &units, &paths).expect("restore state");
            cleanup_failed_release(&journal).expect("cleanup failed release");

            assert_eq!(
                fs::read_link(&current).expect("rolled back current"),
                PathBuf::from("releases/old")
            );
            assert!(!journal.new_release_dir.exists());
            assert!(!journal.install_staging_dir.exists());
            for (index, path) in paths.iter().enumerate() {
                let expected = if index <= failed_index { "old-database" } else { "new-database" };
                assert_eq!(fs::read_to_string(path).expect("database boundary"), expected);
            }
            fs::remove_dir_all(root).expect("cleanup");
        }
    }

    #[cfg(unix)]
    #[test]
    fn interruption_cleanup_preserves_reused_release_directory() {
        let root = std::env::temp_dir().join(format!("rz-reused-release-{}", uuid::Uuid::new_v4()));
        let releases = root.join("releases");
        fs::create_dir_all(releases.join("old")).expect("old release");
        fs::create_dir_all(releases.join("reused")).expect("reused release");
        let current = root.join("current");
        std::os::unix::fs::symlink("releases/old", &current).expect("current");
        let staging = releases.join(".reused.8.installing");
        fs::create_dir(&staging).expect("staging");
        let journal = UpdateJournal {
            release_id: 8,
            backup_dir: root.join("backup"),
            link: current,
            old_target: PathBuf::from("releases/old"),
            new_release_dir: releases.join("reused"),
            install_staging_dir: staging,
            installed_by_update: false,
            stage: "installing".to_string(),
            restarted_units: Vec::new(),
        };
        cleanup_failed_release(&journal).expect("cleanup interruption");
        assert!(journal.new_release_dir.is_dir());
        assert!(!journal.install_staging_dir.exists());
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn four_database_backup_and_restore_are_independent() {
        let root = std::env::temp_dir().join(format!("rz-backup-{}", uuid::Uuid::new_v4()));
        let data = root.join("data");
        let backup = root.join("backup");
        fs::create_dir_all(&data).expect("data dir");
        fs::create_dir_all(&backup).expect("backup dir");
        let paths = ["admin", "monitor", "insights", "reports"]
            .map(|name| data.join(format!("{name}.db")))
            .to_vec();
        for path in &paths {
            let name = path.file_stem().and_then(|value| value.to_str()).expect("name");
            fs::write(path, format!("{name}-database")).expect("database");
            fs::write(PathBuf::from(format!("{}-wal", path.display())), format!("{name}-wal"))
                .expect("wal");
        }

        backup_database_paths(&paths, &backup).expect("backup");
        for path in &paths {
            fs::write(path, "corrupt").expect("corrupt");
            let wal = PathBuf::from(format!("{}-wal", path.display()));
            if wal.exists() {
                fs::remove_file(wal).expect("remove wal");
            }
        }
        restore_database_paths(&paths, &backup).expect("restore");

        for path in &paths {
            let name = path.file_stem().and_then(|value| value.to_str()).expect("name");
            assert_eq!(fs::read_to_string(path).expect("database"), format!("{name}-database"));
            assert!(!PathBuf::from(format!("{}-wal", path.display())).exists());
        }
        fs::remove_dir_all(root).expect("remove backup root");
    }

    #[test]
    fn incomplete_database_backup_never_modifies_live_databases() {
        let root =
            std::env::temp_dir().join(format!("rz-backup-preflight-{}", uuid::Uuid::new_v4()));
        let data = root.join("data");
        let backup = root.join("backup");
        fs::create_dir_all(&data).expect("data dir");
        fs::create_dir_all(&backup).expect("backup dir");
        let paths = ["admin", "monitor", "insights", "reports"]
            .map(|name| data.join(format!("{name}.db")))
            .to_vec();
        for path in &paths {
            fs::write(path, "original").expect("original database");
        }
        backup_database_paths(&paths, &backup).expect("backup");
        fs::remove_file(backup.join("insights.db")).expect("remove one backup");
        for path in &paths {
            fs::write(path, "live-after-update").expect("live database");
        }

        assert!(restore_database_paths(&paths, &backup).is_err());
        for path in &paths {
            assert_eq!(
                fs::read_to_string(path).expect("unchanged live database"),
                "live-after-update"
            );
        }
        fs::remove_dir_all(root).expect("remove backup root");
    }

    #[tokio::test]
    async fn online_sqlite_backup_is_a_readable_consistent_database() {
        let root = std::env::temp_dir().join(format!("rz-online-backup-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).expect("backup root");
        let source = root.join("source.db");
        let destination = root.join("backup.db");
        let pool = crate::infra::db::create_pool_for_path(&source).await.expect("source pool");
        sqlx::query("CREATE TABLE values_table (value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create table");
        sqlx::query("INSERT INTO values_table (value) VALUES ('before')")
            .execute(&pool)
            .await
            .expect("insert value");

        backup_database_online(&source, &destination).await.expect("online backup");
        let backup =
            crate::infra::db::create_pool_for_path(&destination).await.expect("backup pool");
        let value: String = sqlx::query_scalar("SELECT value FROM values_table")
            .fetch_one(&backup)
            .await
            .expect("backup value");
        assert_eq!(value, "before");
        backup.close().await;
        pool.close().await;
        fs::remove_dir_all(root).expect("remove backup root");
    }
}
