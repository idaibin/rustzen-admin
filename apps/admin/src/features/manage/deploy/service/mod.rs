use std::{
    collections::BTreeMap,
    fs,
    future::Future,
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
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

mod database;
mod journal;
mod recovery;
#[cfg(test)]
mod tests;
mod validation;

use database::*;
use journal::*;
use recovery::*;
use validation::*;

pub const DEPLOY_FILE_MAX_SIZE: usize = 256 * 1024 * 1024;
const DEPLOY_BODY_LIMIT: usize = DEPLOY_FILE_MAX_SIZE + 1024 * 1024;
const SYSTEMD_UNITS: &[&str] =
    &["rz-monitor.service", "rz-insights.service", "rz-reports.service", "rz-admin.service"];
const UPDATE_REQUEST_DIR: &str = "update-requests";
const UPDATE_REQUEST_FILE: &str = "pending.json";
const UPDATE_REQUEST_PROCESSING_FILE: &str = ".pending.json.processing";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UpdateRequest {
    release_id: i64,
    deployed_by: String,
}

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
                frontend_hash: bundle.frontend_sha256,
                backend_hash: bundle.backend_sha256,
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
        let detected_arch = bundle.arch.clone();
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
                frontend_hash: bundle.frontend_sha256,
                backend_hash: bundle.backend_sha256,
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
        let runtime_root = fs::canonicalize(CONFIG.runtime_root_dir()).map_err(|error| {
            ServiceError::InvalidOperation(format!("Cannot resolve runtime root: {error}"))
        })?;
        write_update_request(&runtime_root, &UpdateRequest { release_id: id, deployed_by })?;
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
        remove_release_file(&version.file_path)?;
        self.repo.delete_by_id(id).await
    }

    pub async fn cleanup_expired(
        &self,
        _component: Option<DeployComponent>,
    ) -> Result<usize, ServiceError> {
        let versions = self.repo.expired_non_current(Some(&DeployComponent::Release)).await?;
        let mut deleted = 0;
        for version in versions {
            remove_release_file(&version.file_path)?;
            self.repo.delete_by_id(version.id).await?;
            deleted += 1;
        }
        Ok(deleted)
    }

    pub async fn run_update_worker(id: i64) -> Result<(), Box<dyn std::error::Error>> {
        Self::run_update_worker_for(id, None).await
    }

    pub async fn run_update_request_worker() -> Result<(), Box<dyn std::error::Error>> {
        let runtime_root = fs::canonicalize(CONFIG.runtime_root_dir())?;
        let claimed = claim_update_request(&runtime_root)?;
        let request = read_update_request(&claimed);
        fs::remove_file(&claimed)?;
        let request = request?;
        Self::run_update_worker_for(request.release_id, Some(request.deployed_by)).await
    }

    async fn run_update_worker_for(
        id: i64,
        deployed_by: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        recover_interrupted_update().await?;
        let pool = crate::infra::db::create_default_pool().await?;
        crate::infra::db::run_migrations(&pool).await?;
        let service = Self::new(pool.clone());
        let version = service.find_by_id(id).await?;
        ensure_version_is_deployable(&version)?;
        let (data, bundle) = validate_stored_release(&version)?;
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
                    deployed_by.as_deref(),
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

fn update_request_path(runtime_root: &Path) -> PathBuf {
    runtime_root.join("data").join(UPDATE_REQUEST_DIR).join(UPDATE_REQUEST_FILE)
}

fn write_update_request(runtime_root: &Path, request: &UpdateRequest) -> Result<(), ServiceError> {
    let directory = runtime_root.join("data").join(UPDATE_REQUEST_DIR);
    let path = update_request_path(runtime_root);
    if fs::symlink_metadata(&directory)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
        || fs::symlink_metadata(&path)
            .map(|metadata| metadata.file_type().is_symlink())
            .unwrap_or(false)
    {
        return Err(ServiceError::InvalidOperation(
            "Update request path must not be a symlink".into(),
        ));
    }
    let temporary = directory.join(".pending.json.new");
    fs::write(
        &temporary,
        serde_json::to_vec(request)
            .map_err(|error| ServiceError::InvalidOperation(error.to_string()))?,
    )
    .map_err(|error| ServiceError::InvalidOperation(error.to_string()))?;
    fs::rename(temporary, path)
        .map_err(|error| ServiceError::InvalidOperation(error.to_string()))?;
    Ok(())
}

fn claim_update_request(runtime_root: &Path) -> Result<PathBuf, std::io::Error> {
    let directory = runtime_root.join("data").join(UPDATE_REQUEST_DIR);
    let pending = update_request_path(runtime_root);
    let claimed = directory.join(UPDATE_REQUEST_PROCESSING_FILE);
    if fs::symlink_metadata(&claimed).is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "an update request is already being processed",
        ));
    }
    fs::rename(pending, &claimed)?;
    Ok(claimed)
}

fn read_update_request(path: &Path) -> Result<UpdateRequest, Box<dyn std::error::Error>> {
    let mut file = fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() > 16 * 1024 || before.mode() & 0o077 != 0 {
        return Err(std::io::Error::other("update request must be a regular file").into());
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    file.read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    if bytes.len() as u64 != before.len()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.mode() != after.mode()
        || before.len() != after.len()
        || before.ctime() != after.ctime()
        || before.ctime_nsec() != after.ctime_nsec()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
    {
        return Err(std::io::Error::other("update request changed while read").into());
    }
    let request: UpdateRequest = serde_json::from_slice(&bytes)?;
    if request.release_id <= 0 || request.deployed_by.is_empty() || request.deployed_by.len() > 256
    {
        return Err(std::io::Error::other("update request fields are invalid").into());
    }
    Ok(request)
}
