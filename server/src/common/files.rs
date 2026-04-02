use crate::common::error::ServiceError;
use crate::infra::config::CONFIG;

use axum::extract::Multipart;
use std::{fs::File, io::Write, path::PathBuf};
use uuid::Uuid;

const USER_AVATAR_MAX_SIZE: usize = 1024 * 1024;

/// 保存头像
pub async fn save_avatar(multipart: &mut Multipart) -> Result<String, ServiceError> {
    let avatar_dir = &CONFIG.avatar_dir;
    let avatar_public_prefix = format!(
        "{}/avatars",
        CONFIG.upload_public_prefix.trim_end_matches('/')
    );

    // 确保上传目录存在
    tokio::fs::create_dir_all(avatar_dir)
        .await
        .map_err(|_| ServiceError::CreateAvatarFolderFailed)?;

    let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ServiceError::InvalidOperation("Invalid multipart data".into()))?
    else {
        return Err(ServiceError::InvalidOperation("No file provided".into()));
    };

    // 验证文件类型
    let content_type = field.content_type().unwrap_or("");
    if !content_type.starts_with("image/") {
        return Err(ServiceError::InvalidOperation("Only image files are allowed".into()));
    }

    // 获取文件扩展名
    let filename = field.file_name().unwrap_or("unknown");
    let extension = filename.rsplit('.').next().unwrap_or("jpg");

    // 生成唯一文件名
    let file_name = format!("{}.{}", Uuid::new_v4(), extension);
    let file_path = PathBuf::from(avatar_dir).join(&file_name);

    // 读取文件数据
    let data = field
        .bytes()
        .await
        .map_err(|_| ServiceError::InvalidOperation("Failed to read file data".into()))?;

    // 验证文件大小
    if data.len() > USER_AVATAR_MAX_SIZE {
        return Err(ServiceError::InvalidOperation("File size must be less than 1MB".into()));
    }

    // 保存文件
    let mut file = File::create(&file_path).map_err(|_| ServiceError::CreateAvatarFileFailed)?;
    file.write_all(&data).map_err(|_| ServiceError::CreateAvatarFileFailed)?;

    let avatar_url = format!("{}/{}", avatar_public_prefix, file_name);
    tracing::info!("Avatar uploaded successfully: {}", avatar_url);

    Ok(avatar_url)
}
