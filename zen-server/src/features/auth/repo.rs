use super::types::{AuthUserRow, LoginCredentialsRow};
use crate::common::error::ServiceError;

use chrono::Utc;
use sqlx::SqlitePool;

/// Auth db operations.
pub struct AuthRepository;

impl AuthRepository {
    /// Check user by username for authentication (only essential fields)
    pub async fn get_login_credentials(
        pool: &SqlitePool,
        username: &str,
    ) -> Result<Option<LoginCredentialsRow>, ServiceError> {
        sqlx::query_as::<_, LoginCredentialsRow>(
            "SELECT id, password_hash, status, is_system FROM users WHERE username = ? AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error in get_login_credentials, username={}: {:?}",
                username,
                e
            );
            ServiceError::DatabaseQueryFailed
        })
    }

    /// Find user by ID for auth/session data.
    pub async fn find_user_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<AuthUserRow>, ServiceError> {
        sqlx::query_as::<_, AuthUserRow>(
            "SELECT id, username, real_name, email, avatar_url, is_system FROM users WHERE id = ? AND deleted_at IS NULL AND status = 1",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error in find_user_by_id, user_id={}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// Update last login timestamp
    pub async fn update_last_login(pool: &SqlitePool, id: i64) -> Result<(), ServiceError> {
        sqlx::query("UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ?")
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_last_login, user_id={}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(())
    }

    /// Get all permission keys for a user by user ID.
    /// Returns a list of permission strings (e.g., "system:user:list").
    pub async fn get_user_permissions(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<Vec<String>, ServiceError> {
        sqlx::query_scalar("SELECT menu_code FROM user_permissions WHERE user_id = ?")
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Database error in get_user_permissions, user_id={}: {:?}",
                    user_id,
                    e
                );
                ServiceError::DatabaseQueryFailed
            })
    }

    /// Get all permission keys managed by the menu table.
    pub async fn get_all_permissions(pool: &SqlitePool) -> Result<Vec<String>, ServiceError> {
        sqlx::query_scalar("SELECT code FROM menus WHERE deleted_at IS NULL ORDER BY id")
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in get_all_permissions: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })
    }
}
