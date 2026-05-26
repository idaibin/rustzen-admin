use crate::common::error::ServiceError;
use crate::infra::config::CONFIG;

use axum::extract::Multipart;
use std::{fs::File, io::Write};
use uuid::Uuid;

const USER_AVATAR_MAX_SIZE: usize = 1024 * 1024;

/// Saves a user avatar and returns its public URL.
pub async fn save_avatar(multipart: &mut Multipart) -> Result<String, ServiceError> {
    let avatar_dir = CONFIG.avatars_dir();
    let avatar_public_prefix = CONFIG.avatars_prefix();

    tokio::fs::create_dir_all(&avatar_dir)
        .await
        .map_err(|_| ServiceError::CreateAvatarFolderFailed)?;

    let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ServiceError::InvalidOperation("Invalid multipart data".into()))?
    else {
        return Err(ServiceError::InvalidOperation("No file provided".into()));
    };

    let content_type = field
        .content_type()
        .ok_or_else(|| ServiceError::InvalidOperation("Missing file content type".into()))?;
    if !content_type.starts_with("image/") {
        return Err(ServiceError::InvalidOperation("Only image files are allowed".into()));
    }

    let filename = field
        .file_name()
        .ok_or_else(|| ServiceError::InvalidOperation("Missing file name".into()))?;
    let extension = filename
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty())
        .ok_or_else(|| ServiceError::InvalidOperation("Missing file extension".into()))?;

    let file_name = format!("{}.{}", Uuid::new_v4(), extension);
    let file_path = avatar_dir.join(&file_name);

    let data = field
        .bytes()
        .await
        .map_err(|_| ServiceError::InvalidOperation("Failed to read file data".into()))?;

    if data.len() > USER_AVATAR_MAX_SIZE {
        return Err(ServiceError::InvalidOperation("File size must be less than 1MB".into()));
    }

    let mut file = File::create(&file_path).map_err(|_| ServiceError::CreateAvatarFileFailed)?;
    file.write_all(&data).map_err(|_| ServiceError::CreateAvatarFileFailed)?;

    let avatar_url = format!("{}/{}", avatar_public_prefix, file_name);
    tracing::info!("Avatar uploaded successfully: {}", avatar_url);

    Ok(avatar_url)
}
