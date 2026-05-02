use super::types::{PasswordHashRow, UpdateAccountProfileRequest};
use crate::common::error::ServiceError;

use chrono::Utc;
use sqlx::PgPool;

/// Current-account db operations.
pub struct AccountRepository;

impl AccountRepository {
    pub async fn update_avatar(
        pool: &PgPool,
        user_id: i64,
        avatar_url: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query("UPDATE users SET avatar_url = $1, updated_at = $3 WHERE id = $2")
            .bind(avatar_url)
            .bind(user_id)
            .bind(Utc::now().naive_utc())
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_avatar, user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(())
    }

    pub async fn email_exists_for_other_user(
        pool: &PgPool,
        user_id: i64,
        email: &str,
    ) -> Result<bool, ServiceError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id <> $2 AND deleted_at IS NULL)",
        )
        .bind(email)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error in email_exists_for_other_user, user_id={}: {:?}", user_id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn update_profile(
        pool: &PgPool,
        user_id: i64,
        request: &UpdateAccountProfileRequest,
    ) -> Result<(), ServiceError> {
        sqlx::query("UPDATE users SET email = $1, real_name = $2, updated_at = $3 WHERE id = $4 AND deleted_at IS NULL")
            .bind(&request.email)
            .bind(&request.real_name)
            .bind(Utc::now().naive_utc())
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_profile, user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(())
    }

    pub async fn find_password_hash_by_id(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Option<PasswordHashRow>, ServiceError> {
        sqlx::query_as::<_, PasswordHashRow>(
            "SELECT password_hash FROM users WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error in find_password_hash_by_id, user_id={}: {:?}",
                user_id,
                e
            );
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn update_password(
        pool: &PgPool,
        user_id: i64,
        password_hash: &str,
    ) -> Result<(), ServiceError> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3 AND deleted_at IS NULL")
            .bind(password_hash)
            .bind(Utc::now().naive_utc())
            .bind(user_id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_password, user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(())
    }
}
