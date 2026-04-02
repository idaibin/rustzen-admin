use super::types::{AuthUserRow, LoginCredentialsRow};
use crate::common::error::ServiceError;

use chrono::Utc;
use sqlx::PgPool;

/// Auth db for authentication-specific database operations
pub struct AuthRepository;

impl AuthRepository {
    /// Check user by username for authentication (only essential fields)
    pub async fn get_login_credentials(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<LoginCredentialsRow>, ServiceError> {
        sqlx::query_as::<_, LoginCredentialsRow>(
            "SELECT id, password_hash, status, is_system FROM get_login_credentials($1)",
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

    /// Find user by ID for authentication (returns AuthUserRow)
    pub async fn find_user_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<AuthUserRow>, ServiceError> {
        sqlx::query_as::<_, AuthUserRow>(
            "SELECT id, username, real_name, email, avatar_url, is_system FROM get_user_basic_info($1)",
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
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
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
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<String>, ServiceError> {
        sqlx::query_scalar("SELECT get_user_permissions($1)")
            .bind(user_id)
            .fetch_one(pool)
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
    pub async fn get_all_permissions(pool: &PgPool) -> Result<Vec<String>, ServiceError> {
        sqlx::query_scalar("SELECT code FROM menus WHERE deleted_at IS NULL ORDER BY id")
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in get_all_permissions: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })
    }

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
}
