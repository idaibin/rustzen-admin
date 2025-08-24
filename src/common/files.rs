use crate::common::error::ServiceError;

use axum::extract::Multipart;
use std::{fs::File, io::Write};
use uuid::Uuid;

const USER_AVATAR_DIR: &str = "uploads/avatars";
const USER_AVATAR_MAX_SIZE: usize = 1024 * 1024;

/// 保存头像
pub async fn save_avatar(multipart: &mut Multipart) -> Result<String, ServiceError> {
    // 确保上传目录存在
    tokio::fs::create_dir_all(USER_AVATAR_DIR)
        .await
        .map_err(|_| ServiceError::CreateAvatarFolderFailed)?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| ServiceError::InvalidOperation("Invalid multipart data".into()))?
    {
        // 验证文件类型
        let content_type = field.content_type().unwrap_or("");
        if !content_type.starts_with("image/") {
            return Err(
                ServiceError::InvalidOperation("Only image files are allowed".into()).into()
            );
        }

        // 获取文件扩展名
        let filename = field.file_name().unwrap_or("unknown").to_string();
        let extension = filename.split('.').last().unwrap_or("jpg");

        // 生成唯一文件名
        let file_path = format!("{}/{}.{}", USER_AVATAR_DIR, Uuid::new_v4(), extension);

        // 读取文件数据
        let data = field
            .bytes()
            .await
            .map_err(|_| ServiceError::InvalidOperation("Failed to read file data".into()))?;

        // 验证文件大小
        if data.len() > USER_AVATAR_MAX_SIZE {
            return Err(
                ServiceError::InvalidOperation("File size must be less than 1MB".into()).into()
            );
        }

        // 保存文件
        let mut file =
            File::create(&file_path).map_err(|_| ServiceError::CreateAvatarFileFailed)?;
        file.write_all(&data).map_err(|_| ServiceError::CreateAvatarFileFailed)?;

        let avatar_url = format!("/{}", file_path);
        tracing::info!("Avatar uploaded successfully: {}", avatar_url);

        return Ok(avatar_url);
    }

    Err(ServiceError::InvalidOperation("No file provided".into()).into())
}
