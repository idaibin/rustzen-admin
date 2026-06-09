use std::{
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use axum::extract::Multipart;
use sha2::{Digest, Sha256};
use tokio::process::Command;
use zip::ZipArchive;

use crate::{
    common::{
        api::ApiResponse,
        error::ServiceError,
        pagination::{Pagination, PaginationQuery},
    },
    features::manage::deploy::types::{
        DeployComponent, DeployVersionRequest, DeploymentItem, DeploymentPayload,
        ExpireVersionRequest, ListDeploymentsQuery,
    },
    infra::config::CONFIG,
};

use super::repo::DeployRepository;

pub const DEPLOY_FILE_MAX_SIZE: usize = 64 * 1024 * 1024;
const DEPLOY_BODY_LIMIT: usize = DEPLOY_FILE_MAX_SIZE + 1024 * 1024;
const SERVER_BINARY_NAME: &str = "rustzen-admin";
const SERVER_SYSTEMD_UNIT: &str = "rustzen-admin.service";
const WEB_MARKER_FILE: &str = "dist/__rustzen_admin_marker__.json";
const SERVER_MARKER_PREFIX: &[u8] = b"RUSTZEN_ADMIN_MARKER\ncomponent=server\n";

#[derive(Clone)]
pub struct DeployService {
    repo: Arc<DeployRepository>,
}

impl DeployService {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self {
            repo: Arc::new(DeployRepository::new(pool)),
        }
    }

    pub fn upload_body_limit() -> usize {
        DEPLOY_BODY_LIMIT
    }

    pub async fn list(
        &self,
        query: ListDeploymentsQuery,
    ) -> Result<ApiResponse<Vec<DeploymentItem>>, ServiceError> {
        let pagination = Pagination::from_query(PaginationQuery {
            current: query.current,
            page_size: query.page_size,
        });
        let (items, total) = self
            .repo
            .list(&query, pagination.offset.into(), pagination.limit.into())
            .await?;
        Ok(ApiResponse::new(items, Some(total)))
    }

    pub async fn upload(&self, mut multipart: Multipart) -> Result<DeploymentItem, ServiceError> {
        let mut component = None;
        let mut version = None;
        let mut arch = None;
        let mut notes = None;
        let mut file_data = None;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|_| ServiceError::InvalidOperation("Invalid multipart data".to_string()))?
        {
            let Some(name) = field.name().map(str::to_string) else {
                continue;
            };

            match name.as_str() {
                "component" => {
                    component = Some(parse_component(
                        field.text().await.map_err(|_| {
                            ServiceError::InvalidOperation("Invalid component field".to_string())
                        })?,
                    )?);
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
                "notes" => {
                    notes = normalize_optional(field.text().await.unwrap_or_default());
                }
                "file" => {
                    let data = field.bytes().await.map_err(|_| {
                        ServiceError::InvalidOperation("Failed to read uploaded file".to_string())
                    })?;
                    file_data = Some(data.to_vec());
                }
                _ => {}
            }
        }

        let component =
            component.ok_or_else(|| ServiceError::InvalidOperation("component is required".into()))?;
        let version =
            version.ok_or_else(|| ServiceError::InvalidOperation("version is required".into()))?;
        let file_data =
            file_data.ok_or_else(|| ServiceError::InvalidOperation("file is required".into()))?;
        validate_upload_size(&file_data)?;

        let arch = match component {
            DeployComponent::Server => {
                let arch = normalize_arch(
                    arch.ok_or_else(|| {
                        ServiceError::InvalidOperation("server component requires arch".into())
                    })?
                    .as_str(),
                )?;
                validate_server_file(&file_data, &arch)?;
                arch
            }
            DeployComponent::Web => {
                validate_web_file(&file_data)?;
                "x86_64".to_string()
            }
        };

        if self.repo.version_exists(&component, &version, &arch).await? {
            return Err(ServiceError::InvalidOperation(
                "Deploy version already exists for this component and arch".to_string(),
            ));
        }

        let file_hash = sha256_hex(&file_data);
        let file_path = save_version_file(&component, &version, &arch, &file_data).await?;

        self.repo
            .insert(&DeploymentPayload {
                component,
                version,
                arch,
                file_path: file_path.to_string_lossy().to_string(),
                file_size: file_data.len() as i64,
                file_hash,
                notes,
            })
            .await
    }

    pub async fn find_by_id(&self, id: i64) -> Result<DeploymentItem, ServiceError> {
        self.repo.find_by_id(id).await
    }

    pub async fn deploy(
        &self,
        id: i64,
        request: DeployVersionRequest,
    ) -> Result<bool, ServiceError> {
        if let Some(version_id) = request.version_id
            && version_id != id
        {
            return Err(ServiceError::InvalidOperation(
                "versionId must match the deployment id".to_string(),
            ));
        }

        let version = self.repo.find_by_id(id).await?;
        validate_stored_file(&version)?;

        match version.component {
            DeployComponent::Server => self.deploy_server(&version, &request).await?,
            DeployComponent::Web => self.deploy_web(&version, &request).await?,
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
                "Cannot delete the current deploy version".to_string(),
            ));
        }
        remove_deploy_file(&version)?;
        self.repo.delete_by_id(id).await?;
        Ok(version)
    }

    pub async fn cleanup_expired(
        &self,
        component: Option<DeployComponent>,
    ) -> Result<usize, ServiceError> {
        let versions = self.repo.expired_non_current(component.as_ref()).await?;
        let mut deleted = 0;

        for version in versions {
            match remove_deploy_file(&version) {
                Ok(()) => {
                    self.repo.delete_by_id(version.id).await?;
                    deleted += 1;
                }
                Err(err) => {
                    tracing::warn!(
                        "Skipped deploy cleanup for id={} file={}: {}",
                        version.id,
                        version.file_path,
                        err
                    );
                }
            }
        }

        Ok(deleted)
    }

    async fn deploy_server(
        &self,
        version: &DeploymentItem,
        request: &DeployVersionRequest,
    ) -> Result<(), ServiceError> {
        let target_bin = runtime_root_dir().join("bin").join(SERVER_BINARY_NAME);
        fs::create_dir_all(parent_dir(&target_bin)?).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to create bin directory: {err}"))
        })?;

        if let Some(current) = self
            .repo
            .find_current(&version.component, &version.arch)
            .await?
            && current.id == version.id
            && fs::read_link(&target_bin)
                .map(|target| target == Path::new(&version.file_path))
                .unwrap_or(false)
        {
            return Ok(());
        }

        prepare_server_restart().await?;

        let old_target = swap_symlink(&target_bin, Path::new(&version.file_path)).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to switch server binary: {err}"))
        })?;

        if let Err(err) = restart_server().await {
            if let Err(restore_err) = restore_symlink(&target_bin, old_target.as_deref()) {
                tracing::error!("Failed to restore server symlink: {}", restore_err);
            }
            return Err(err);
        }

        if let Err(err) = self
            .repo
            .set_current(
                &version.component,
                &version.arch,
                version.id,
                request.deployed_by.as_deref(),
            )
            .await
        {
            if let Err(restore_err) = restore_symlink(&target_bin, old_target.as_deref()) {
                tracing::error!("Failed to restore server symlink: {}", restore_err);
            }
            return Err(err);
        }

        Ok(())
    }

    async fn deploy_web(
        &self,
        version: &DeploymentItem,
        request: &DeployVersionRequest,
    ) -> Result<(), ServiceError> {
        if let Some(current) = self.repo.find_current(&version.component, &version.arch).await?
            && current.id == version.id
        {
            return Ok(());
        }

        let web_root = runtime_root_dir().join("web");
        let tmp_dir = web_root.join(format!(".deploy-{}", version.id));
        if tmp_dir.exists() {
            fs::remove_dir_all(&tmp_dir).map_err(|err| {
                ServiceError::InvalidOperation(format!("Failed to clear temp web directory: {err}"))
            })?;
        }
        fs::create_dir_all(&tmp_dir).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to create temp web directory: {err}"))
        })?;

        let extract_result = extract_zip_to_dir(Path::new(&version.file_path), &tmp_dir);
        if let Err(err) = extract_result {
            let _ = fs::remove_dir_all(&tmp_dir);
            return Err(err);
        }

        let extracted_dist = tmp_dir.join("dist");
        if !extracted_dist.join("index.html").is_file() {
            let _ = fs::remove_dir_all(&tmp_dir);
            return Err(ServiceError::InvalidOperation(
                "web package must contain dist/index.html".to_string(),
            ));
        }

        fs::create_dir_all(&web_root).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to create web directory: {err}"))
        })?;
        let dist_dir = web_root.join("dist");
        let new_dist = web_root.join("dist.new");
        let prev_dist = web_root.join("dist.prev");
        remove_path_if_exists(&new_dist)?;
        fs::rename(&extracted_dist, &new_dist).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to prepare new web dist: {err}"))
        })?;
        remove_path_if_exists(&prev_dist)?;
        if dist_dir.exists() {
            fs::rename(&dist_dir, &prev_dist).map_err(|err| {
                ServiceError::InvalidOperation(format!("Failed to archive previous web dist: {err}"))
            })?;
        }
        fs::rename(&new_dist, &dist_dir).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to activate web dist: {err}"))
        })?;
        let _ = fs::remove_dir_all(&tmp_dir);

        if let Err(err) = self
            .repo
            .set_current(
                &version.component,
                &version.arch,
                version.id,
                request.deployed_by.as_deref(),
            )
            .await
        {
            if let Err(restore_err) = restore_web_dist(&dist_dir, &prev_dist) {
                tracing::error!("Failed to rollback web dist after database error: {}", restore_err);
            }
            return Err(err);
        }

        let _ = remove_path_if_exists(&prev_dist);
        Ok(())
    }
}

async fn save_version_file(
    component: &DeployComponent,
    version: &str,
    arch: &str,
    file_data: &[u8],
) -> Result<PathBuf, ServiceError> {
    let path = version_file_path(component, version, arch)?;
    if path.exists() {
        return Err(ServiceError::InvalidOperation(format!(
            "Deploy file already exists: {}",
            path.display()
        )));
    }
    fs::create_dir_all(parent_dir(&path)?).map_err(|err| {
        ServiceError::InvalidOperation(format!("Failed to create version directory: {err}"))
    })?;
    tokio::fs::write(&path, file_data).await.map_err(|err| {
        ServiceError::InvalidOperation(format!("Failed to save deploy file: {err}"))
    })?;

    if matches!(component, DeployComponent::Server) {
        set_executable(&path)?;
    }

    Ok(path)
}

fn version_file_path(
    component: &DeployComponent,
    version: &str,
    arch: &str,
) -> Result<PathBuf, ServiceError> {
    let root = runtime_root_dir();
    match component {
        DeployComponent::Server => Ok(root
            .join("versions")
            .join(format!("server-{version}-{arch}"))),
        DeployComponent::Web => Ok(root.join("web").join(format!("web-{version}.zip"))),
    }
}

fn validate_stored_file(version: &DeploymentItem) -> Result<(), ServiceError> {
    let data = fs::read(&version.file_path).map_err(|err| {
        ServiceError::InvalidOperation(format!("Failed to read deploy file: {err}"))
    })?;
    if data.len() as i64 != version.file_size {
        return Err(ServiceError::InvalidOperation(
            "Deploy file size does not match database record".to_string(),
        ));
    }
    let actual_hash = sha256_hex(&data);
    if actual_hash != version.file_hash {
        return Err(ServiceError::InvalidOperation(
            "Deploy file hash does not match database record".to_string(),
        ));
    }

    match version.component {
        DeployComponent::Server => validate_server_file(&data, &version.arch),
        DeployComponent::Web => validate_web_file(&data),
    }
}

fn validate_upload_size(file_data: &[u8]) -> Result<(), ServiceError> {
    if file_data.is_empty() {
        return Err(ServiceError::InvalidOperation("file is required".to_string()));
    }
    if file_data.len() > DEPLOY_FILE_MAX_SIZE {
        return Err(ServiceError::InvalidOperation(format!(
            "file must be smaller than {} MB",
            DEPLOY_FILE_MAX_SIZE / 1024 / 1024
        )));
    }
    Ok(())
}

fn validate_server_file(file_data: &[u8], expected_arch: &str) -> Result<(), ServiceError> {
    if is_zip(file_data) {
        return Err(ServiceError::InvalidOperation(
            "server component does not accept zip files".to_string(),
        ));
    }
    if !(is_elf(file_data) || is_macho_or_fat(file_data)) {
        return Err(ServiceError::InvalidOperation(
            "server file must be an executable binary".to_string(),
        ));
    }
    if !file_data
        .windows(SERVER_MARKER_PREFIX.len())
        .any(|window| window == SERVER_MARKER_PREFIX)
    {
        return Err(ServiceError::InvalidOperation(
            "server file marker check failed".to_string(),
        ));
    }
    if let Some(detected_arch) = detect_binary_arch(file_data)?
        && detected_arch != expected_arch
    {
        return Err(ServiceError::InvalidOperation(format!(
            "server file arch mismatch: detected {detected_arch}, expected {expected_arch}"
        )));
    }
    Ok(())
}

fn validate_web_file(file_data: &[u8]) -> Result<(), ServiceError> {
    const MAX_FILES: usize = 5000;
    const MAX_SINGLE_UNCOMPRESSED: u64 = 5 * 1024 * 1024;
    const MAX_TOTAL_UNCOMPRESSED: u64 = 50 * 1024 * 1024;

    if !is_zip(file_data) {
        return Err(ServiceError::InvalidOperation(
            "web component only accepts zip files".to_string(),
        ));
    }

    let mut archive = ZipArchive::new(Cursor::new(file_data))
        .map_err(|_| ServiceError::InvalidOperation("web zip cannot be parsed".to_string()))?;
    let file_count = archive.len();
    if file_count == 0 || file_count > MAX_FILES {
        return Err(ServiceError::InvalidOperation(format!(
            "web zip file count is invalid: {file_count}"
        )));
    }

    let mut has_index = false;
    let mut has_asset = false;
    let mut has_marker = false;
    let mut total_uncompressed = 0_u64;

    for index in 0..file_count {
        let mut file = archive
            .by_index(index)
            .map_err(|_| ServiceError::InvalidOperation("web zip read failed".to_string()))?;
        let Some(enclosed_name) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            return Err(ServiceError::InvalidOperation(
                "web zip contains invalid paths".to_string(),
            ));
        };
        let Some(name) = zip_path_name(&enclosed_name) else {
            return Err(ServiceError::InvalidOperation(
                "web zip contains invalid paths".to_string(),
            ));
        };

        if name == "dist/index.html" {
            has_index = true;
        }
        if name.starts_with("dist/assets/")
            && (name.ends_with(".js") || name.ends_with(".css"))
        {
            has_asset = true;
        }
        if name == WEB_MARKER_FILE {
            has_marker = true;
            validate_web_marker(&mut file)?;
        }

        let size = file.size();
        if size > MAX_SINGLE_UNCOMPRESSED {
            return Err(ServiceError::InvalidOperation(format!(
                "web zip file is too large after unzip: {name}"
            )));
        }
        total_uncompressed = total_uncompressed.saturating_add(size);
        if total_uncompressed > MAX_TOTAL_UNCOMPRESSED {
            return Err(ServiceError::InvalidOperation(
                "web zip total uncompressed size is too large".to_string(),
            ));
        }
    }

    if !has_index {
        return Err(ServiceError::InvalidOperation(
            "web zip must contain dist/index.html".to_string(),
        ));
    }
    if !has_asset {
        return Err(ServiceError::InvalidOperation(
            "web zip must contain dist/assets/*.js or *.css".to_string(),
        ));
    }
    if !has_marker {
        return Err(ServiceError::InvalidOperation(format!(
            "web zip must contain {WEB_MARKER_FILE}"
        )));
    }

    Ok(())
}

fn validate_web_marker<R: Read>(reader: &mut R) -> Result<(), ServiceError> {
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .map_err(|_| ServiceError::InvalidOperation("web marker cannot be read".to_string()))?;
    let marker: serde_json::Value = serde_json::from_str(&content)
        .map_err(|_| ServiceError::InvalidOperation("web marker is not valid JSON".to_string()))?;

    if marker.get("component").and_then(|value| value.as_str()) != Some("web") {
        return Err(ServiceError::InvalidOperation(
            "web marker component must be web".to_string(),
        ));
    }
    if marker.get("build_id").and_then(|value| value.as_str()) != Some("manual") {
        return Err(ServiceError::InvalidOperation(
            "web marker build_id must be manual".to_string(),
        ));
    }
    Ok(())
}

fn extract_zip_to_dir(zip_path: &Path, output_dir: &Path) -> Result<(), ServiceError> {
    let file = fs::File::open(zip_path)
        .map_err(|err| ServiceError::InvalidOperation(format!("Failed to open web zip: {err}")))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|_| ServiceError::InvalidOperation("web zip cannot be parsed".to_string()))?;

    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|_| ServiceError::InvalidOperation("web zip read failed".to_string()))?;
        let Some(enclosed_name) = file.enclosed_name().map(|path| path.to_path_buf()) else {
            return Err(ServiceError::InvalidOperation(
                "web zip contains invalid paths".to_string(),
            ));
        };
        let output_path = output_dir.join(enclosed_name);
        if file.is_dir() {
            fs::create_dir_all(&output_path).map_err(|err| {
                ServiceError::InvalidOperation(format!("Failed to create web directory: {err}"))
            })?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                ServiceError::InvalidOperation(format!("Failed to create web directory: {err}"))
            })?;
        }
        let mut output = fs::File::create(&output_path).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to create web file: {err}"))
        })?;
        std::io::copy(&mut file, &mut output).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to extract web file: {err}"))
        })?;
    }

    Ok(())
}

fn parse_component(value: String) -> Result<DeployComponent, ServiceError> {
    match value.trim() {
        "server" => Ok(DeployComponent::Server),
        "web" => Ok(DeployComponent::Web),
        _ => Err(ServiceError::InvalidOperation(
            "component must be server or web".to_string(),
        )),
    }
}

fn validate_version(value: String) -> Result<String, ServiceError> {
    let value = non_empty(value, "version")?;
    if value.len() > 64 {
        return Err(ServiceError::InvalidOperation(
            "version must be at most 64 characters".to_string(),
        ));
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        Ok(value)
    } else {
        Err(ServiceError::InvalidOperation(
            "version can only contain letters, numbers, dot, underscore, and dash".to_string(),
        ))
    }
}

fn normalize_arch(value: &str) -> Result<String, ServiceError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "x86_64" | "amd64" => Ok("x86_64".to_string()),
        "aarch64" | "arm64" => Ok("aarch64".to_string()),
        _ => Err(ServiceError::InvalidOperation(
            "arch must be x86_64 or aarch64".to_string(),
        )),
    }
}

fn detect_binary_arch(file_data: &[u8]) -> Result<Option<String>, ServiceError> {
    if is_elf(file_data) {
        return detect_elf_arch(file_data);
    }
    if is_macho_or_fat(file_data) {
        return detect_macho_arch(file_data);
    }
    Ok(None)
}

fn detect_elf_arch(file_data: &[u8]) -> Result<Option<String>, ServiceError> {
    if file_data.len() < 20 {
        return Ok(None);
    }

    let is_little_endian = file_data[5] == 1;
    let machine = if is_little_endian {
        u16::from_le_bytes([file_data[18], file_data[19]])
    } else {
        u16::from_be_bytes([file_data[18], file_data[19]])
    };

    match machine {
        62 => Ok(Some("x86_64".to_string())),
        183 => Ok(Some("aarch64".to_string())),
        _ => Ok(None),
    }
}

fn detect_macho_arch(file_data: &[u8]) -> Result<Option<String>, ServiceError> {
    if file_data.len() < 8 {
        return Ok(None);
    }

    let magic_be = u32::from_be_bytes([file_data[0], file_data[1], file_data[2], file_data[3]]);
    let magic_le = u32::from_le_bytes([file_data[0], file_data[1], file_data[2], file_data[3]]);
    if magic_be == 0xCAFEBABE || magic_le == 0xBEBAFECA {
        return Ok(None);
    }

    let little_endian = matches!(magic_le, 0xFEEDFACE | 0xFEEDFACF);
    let cputype = if little_endian {
        u32::from_le_bytes([file_data[4], file_data[5], file_data[6], file_data[7]])
    } else {
        u32::from_be_bytes([file_data[4], file_data[5], file_data[6], file_data[7]])
    };

    const CPU_TYPE_X86_64: u32 = 0x01000007;
    const CPU_TYPE_ARM64: u32 = 0x0100000C;
    match cputype {
        CPU_TYPE_X86_64 => Ok(Some("x86_64".to_string())),
        CPU_TYPE_ARM64 => Ok(Some("aarch64".to_string())),
        _ => Ok(None),
    }
}

fn swap_symlink(target_link: &Path, new_target: &Path) -> std::io::Result<Option<PathBuf>> {
    let old_target = if target_link.exists() {
        fs::read_link(target_link).ok()
    } else {
        None
    };
    if target_link.exists() && old_target.is_none() {
        fs::remove_file(target_link)?;
    }
    let parent = parent_dir(target_link)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err.to_string()))?;
    let tmp_link = parent.join(format!(
        ".{}.tmp",
        target_link
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("rustzen-admin")
    ));
    if tmp_link.exists() {
        let _ = fs::remove_file(&tmp_link);
    }
    std::os::unix::fs::symlink(new_target, &tmp_link)?;
    fs::rename(&tmp_link, target_link)?;
    Ok(old_target)
}

fn restore_symlink(target_link: &Path, old_target: Option<&Path>) -> std::io::Result<()> {
    if target_link.exists() {
        let _ = fs::remove_file(target_link);
    }
    let Some(old_target) = old_target else {
        return Ok(());
    };
    let parent = target_link
        .parent()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing parent"))?;
    let tmp_link = parent.join(format!(
        ".{}.tmp",
        target_link
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("rustzen-admin")
    ));
    if tmp_link.exists() {
        let _ = fs::remove_file(&tmp_link);
    }
    std::os::unix::fs::symlink(old_target, &tmp_link)?;
    fs::rename(&tmp_link, target_link)?;
    Ok(())
}

fn restore_web_dist(dist_dir: &Path, prev_dist: &Path) -> Result<(), ServiceError> {
    if !prev_dist.exists() {
        remove_path_if_exists(dist_dir)?;
        return Ok(());
    }

    let failed_dist = dist_dir.with_file_name("dist.failed");
    remove_path_if_exists(&failed_dist)?;
    if dist_dir.exists() {
        fs::rename(dist_dir, &failed_dist).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to archive failed web dist: {err}"))
        })?;
    }

    if let Err(err) = fs::rename(prev_dist, dist_dir) {
        if failed_dist.exists()
            && let Err(restore_err) = fs::rename(&failed_dist, dist_dir)
        {
            tracing::error!("Failed to restore failed web dist: {}", restore_err);
        }
        return Err(ServiceError::InvalidOperation(format!(
            "Failed to restore previous web dist: {err}"
        )));
    }

    let _ = remove_path_if_exists(&failed_dist);
    Ok(())
}

async fn restart_server() -> Result<(), ServiceError> {
    let command = format!("sleep 1; exec systemctl restart {SERVER_SYSTEMD_UNIT}");
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to trigger server restart: {err}"))
        })?;
    Ok(())
}

async fn prepare_server_restart() -> Result<(), ServiceError> {
    let status = Command::new("systemctl")
        .arg("daemon-reload")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to prepare server restart: {err}"))
        })?;

    if !status.success() {
        return Err(ServiceError::InvalidOperation(
            "Failed to prepare server restart: systemctl daemon-reload failed".to_string(),
        ));
    }

    Ok(())
}

fn set_executable(path: &Path) -> Result<(), ServiceError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)
            .map_err(|err| {
                ServiceError::InvalidOperation(format!("Failed to read file metadata: {err}"))
            })?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|err| {
            ServiceError::InvalidOperation(format!("Failed to set file permissions: {err}"))
        })?;
    }
    Ok(())
}

fn runtime_root_dir() -> PathBuf {
    resolve_runtime_path(&CONFIG.runtime_root_dir())
}

fn resolve_runtime_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }
}

fn parent_dir(path: &Path) -> Result<&Path, ServiceError> {
    path.parent().ok_or_else(|| {
        ServiceError::InvalidOperation(format!("Path has no parent: {}", path.display()))
    })
}

fn remove_path_if_exists(path: &Path) -> Result<(), ServiceError> {
    if !path.exists() {
        return Ok(());
    }
    let metadata = fs::symlink_metadata(path).map_err(|err| {
        ServiceError::InvalidOperation(format!("Failed to read path metadata: {err}"))
    })?;
    if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
    .map_err(|err| ServiceError::InvalidOperation(format!("Failed to remove path: {err}")))
}

fn is_zip(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == b'P' && bytes[1] == b'K'
}

fn is_elf(bytes: &[u8]) -> bool {
    bytes.len() >= 4 && bytes[0] == 0x7F && bytes[1] == b'E' && bytes[2] == b'L' && bytes[3] == b'F'
}

fn is_macho_or_fat(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let be = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let le = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    matches!(be, 0xFEEDFACE | 0xFEEDFACF | 0xCAFEBABE | 0xBEBAFECA)
        || matches!(le, 0xFEEDFACE | 0xFEEDFACF | 0xCAFEBABE | 0xBEBAFECA)
}

fn zip_path_name(path: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        let std::path::Component::Normal(part) = component else {
            return None;
        };
        parts.push(part.to_str()?.to_string());
    }
    Some(parts.join("/"))
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>()
}

fn non_empty(value: String, field: &str) -> Result<String, ServiceError> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(ServiceError::InvalidOperation(format!("{field} is required")));
    }
    Ok(value)
}

fn normalize_optional(value: String) -> Option<String> {
    let value = value.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn remove_deploy_file(version: &DeploymentItem) -> Result<(), ServiceError> {
    let path = Path::new(&version.file_path);
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(path).map_err(|err| {
        ServiceError::InvalidOperation(format!("Failed to remove deploy version file: {err}"))
    })
}
