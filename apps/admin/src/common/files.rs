use crate::common::error::ServiceError;
use crate::infra::config::CONFIG;

use axum::{extract::Multipart, http::StatusCode};
use image::{ImageFormat, ImageReader, Limits};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};
use tokio::{io::AsyncWriteExt, sync::Semaphore};
use uuid::Uuid;

const USER_AVATAR_MAX_SIZE: usize = 1024 * 1024;
const USER_AVATAR_MAX_DIMENSION: u32 = 2048;
const USER_AVATAR_MAX_DECODED_SIZE: u64 = 20 * 1024 * 1024;
const USER_AVATAR_MAX_CONCURRENT_DECODES: usize = 2;
static AVATAR_DECODE_SLOTS: Semaphore = Semaphore::const_new(USER_AVATAR_MAX_CONCURRENT_DECODES);

/// Saves a user avatar and returns its public URL.
pub async fn save_avatar(multipart: &mut Multipart) -> Result<String, ServiceError> {
    let avatar_dir = CONFIG.avatars_dir();
    let avatar_public_prefix = CONFIG.avatars_prefix();

    tokio::fs::create_dir_all(&avatar_dir)
        .await
        .map_err(|_| ServiceError::CreateAvatarFolderFailed)?;

    let Some(field) = multipart.next_field().await.map_err(|error| {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE
            || error.body_text() == "Request payload is too large"
        {
            ServiceError::PayloadTooLarge
        } else {
            ServiceError::InvalidOperation("Invalid multipart data".into())
        }
    })?
    else {
        return Err(ServiceError::InvalidOperation("No file provided".into()));
    };

    let content_type = field
        .content_type()
        .ok_or_else(|| ServiceError::InvalidOperation("Missing file content type".into()))?;
    if !content_type.starts_with("image/") {
        return Err(ServiceError::InvalidOperation("Only image files are allowed".into()));
    }

    field.file_name().ok_or_else(|| ServiceError::InvalidOperation("Missing file name".into()))?;

    let data = field.bytes().await.map_err(|error| {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE
            || error.body_text() == "Request payload is too large"
        {
            ServiceError::PayloadTooLarge
        } else {
            ServiceError::InvalidOperation("Failed to read file data".into())
        }
    })?;

    let _decode_permit = AVATAR_DECODE_SLOTS
        .acquire()
        .await
        .map_err(|_| ServiceError::InvalidOperation("Failed to validate image".into()))?;
    let (data, extension) = tokio::task::spawn_blocking(move || {
        validate_avatar_image(&data).map(|extension| (data, extension))
    })
    .await
    .map_err(|_| ServiceError::InvalidOperation("Failed to validate image".into()))??;
    let file_name = format!("{}.{}", Uuid::new_v4(), extension);
    write_avatar_atomically(&avatar_dir, &file_name, &data).await?;

    let avatar_url = format!("{}/{}", avatar_public_prefix, file_name);
    tracing::info!("Avatar uploaded successfully: {}", avatar_url);

    Ok(avatar_url)
}

async fn write_avatar_atomically(
    avatar_dir: &Path,
    file_name: &str,
    data: &[u8],
) -> Result<(), ServiceError> {
    let file_path = avatar_dir.join(file_name);
    let temporary_path = avatar_dir.join(format!(".upload-{}.tmp", Uuid::new_v4()));
    let result = async {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
            .await?;
        file.write_all(data).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&temporary_path, &file_path).await
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(&temporary_path).await;
        return Err(ServiceError::CreateAvatarFileFailed);
    }
    Ok(())
}

fn validate_avatar_image(data: &[u8]) -> Result<&'static str, ServiceError> {
    let invalid = || ServiceError::InvalidOperation("Upload a valid PNG or JPEG image".into());
    if data.len() > USER_AVATAR_MAX_SIZE {
        return Err(ServiceError::PayloadTooLarge);
    }
    let mut reader =
        ImageReader::new(Cursor::new(data)).with_guessed_format().map_err(|_| invalid())?;
    let extension = match reader.format() {
        Some(ImageFormat::Png) => "png",
        Some(ImageFormat::Jpeg) => "jpg",
        _ => return Err(invalid()),
    };
    let mut limits = Limits::default();
    limits.max_image_width = Some(USER_AVATAR_MAX_DIMENSION);
    limits.max_image_height = Some(USER_AVATAR_MAX_DIMENSION);
    limits.max_alloc = Some(USER_AVATAR_MAX_DECODED_SIZE);
    reader.limits(limits);
    reader.decode().map_err(|_| invalid())?;
    Ok(extension)
}

/// Removes a saved avatar by its public URL.
pub async fn remove_avatar_by_url(avatar_url: &str) -> Result<(), std::io::Error> {
    let Some(file_path) = avatar_file_path(avatar_url) else {
        return Ok(());
    };
    match tokio::fs::remove_file(file_path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn avatar_file_path(avatar_url: &str) -> Option<PathBuf> {
    let prefix = format!("{}/", CONFIG.avatars_prefix().trim_end_matches('/'));
    let file_name = avatar_url.strip_prefix(&prefix)?;
    if file_name.contains('/') || file_name.contains('\\') || file_name.is_empty() {
        return None;
    }
    Some(CONFIG.avatars_dir().join(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatars_require_decodable_png_or_jpeg_with_a_content_derived_extension() {
        assert!(validate_avatar_image(b"this is not a PNG").is_err());
        for (format, extension) in [(ImageFormat::Png, "png"), (ImageFormat::Jpeg, "jpg")] {
            let mut encoded = Cursor::new(Vec::new());
            image::RgbImage::new(2, 2).write_to(&mut encoded, format).unwrap();
            let data = encoded.into_inner();
            assert_eq!(validate_avatar_image(&data).unwrap(), extension);
            assert!(validate_avatar_image(&data[..data.len() / 2]).is_err());
        }
        assert!(validate_avatar_image(b"<svg xmlns='http://www.w3.org/2000/svg'/>").is_err());
    }

    #[test]
    fn avatars_reject_payloads_larger_than_one_megabyte_before_decoding() {
        let error = validate_avatar_image(&vec![0; USER_AVATAR_MAX_SIZE + 1]).unwrap_err();
        assert!(matches!(error, ServiceError::PayloadTooLarge));
    }

    #[test]
    fn avatars_reject_dimensions_that_can_amplify_small_uploads() {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbImage::new(USER_AVATAR_MAX_DIMENSION + 1, 1)
            .write_to(&mut encoded, ImageFormat::Png)
            .unwrap();
        assert!(validate_avatar_image(&encoded.into_inner()).is_err());
    }

    #[tokio::test]
    async fn avatar_decode_concurrency_is_bounded() {
        let semaphore = Semaphore::new(USER_AVATAR_MAX_CONCURRENT_DECODES);
        let _first = semaphore.acquire().await.unwrap();
        let _second = semaphore.acquire().await.unwrap();
        assert!(semaphore.try_acquire().is_err());
    }

    #[tokio::test]
    async fn avatar_writes_publish_only_complete_files_and_clean_failed_temporary_files() {
        let directory = std::env::temp_dir().join(format!("rustzen-avatar-{}", Uuid::new_v4()));
        tokio::fs::create_dir(&directory).await.unwrap();

        write_avatar_atomically(&directory, "complete.png", b"complete").await.unwrap();
        assert_eq!(tokio::fs::read(directory.join("complete.png")).await.unwrap(), b"complete");

        tokio::fs::create_dir(directory.join("occupied.png")).await.unwrap();
        assert!(write_avatar_atomically(&directory, "occupied.png", b"partial").await.is_err());
        let mut entries = tokio::fs::read_dir(&directory).await.unwrap();
        while let Some(entry) = entries.next_entry().await.unwrap() {
            assert!(!entry.file_name().to_string_lossy().starts_with(".upload-"));
        }

        tokio::fs::remove_dir_all(directory).await.unwrap();
    }
}
