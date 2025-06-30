use super::model::{AuthMenuInfoEntity, AuthUserInfo, LoginCredentialsEntity};
use sqlx::PgPool;

use chrono::Utc;

/// Auth repository for authentication-specific database operations
pub struct AuthRepository;

impl AuthRepository {
    /// Check user by username for authentication (only essential fields)
    pub async fn get_login_credentials(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<LoginCredentialsEntity>, sqlx::Error> {
        tracing::debug!("Querying login credentials for username: {}", username);

        let user = sqlx::query_as::<_, LoginCredentialsEntity>(
            "SELECT id, username, password_hash, status, is_super_admin FROM users WHERE username = $1 AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error in get_login_credentials, username={}: {:?}", username, e);
            e
        })?;

        if user.is_some() {
            tracing::debug!("User found for username: {}", username);
        } else {
            tracing::debug!("No user found for username: {}", username);
        }

        Ok(user)
    }

    /// Find user by ID for authentication (returns AuthUserInfo)
    /// Optimized version using the helper function from 004_user_info_optimization.sql
    pub async fn get_user_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<AuthUserInfo>, sqlx::Error> {
        tracing::debug!("Querying user basic info for user_id: {}", id);

        let user = sqlx::query_as::<_, AuthUserInfo>(
            "SELECT id, username, real_name, avatar_url FROM get_user_basic_info($1)",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error in get_user_by_id, user_id={}: {:?}", id, e);
            e
        })?;

        if user.is_some() {
            tracing::debug!("User basic info found for user_id: {}", id);
        } else {
            tracing::warn!("No user found for user_id: {}", id);
        }

        Ok(user)
    }

    /// Update last login timestamp
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        tracing::debug!("Updating last login timestamp for user_id: {}", id);

        let result =
            sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
                .bind(Utc::now().naive_utc())
                .bind(id)
                .execute(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Database error in update_last_login, user_id={}: {:?}", id, e);
                    e
                })?;

        if result.rows_affected() == 0 {
            tracing::warn!("No rows affected when updating last login for user_id: {}", id);
        } else {
            tracing::debug!("Successfully updated last login for user_id: {}", id);
        }

        Ok(())
    }

    /// Get all permission keys for a user by user ID.
    /// Returns a list of permission strings (e.g., "system:user:list").
    pub async fn get_user_permissions(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        tracing::debug!("Querying permissions for user_id: {}", user_id);

        // Use the optimized user_permissions view
        let permissions = sqlx::query_scalar::<_, String>(
            r#"
                SELECT DISTINCT permission_code
                FROM user_permissions
                WHERE user_id = $1
                ORDER BY permission_code
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error in get_user_permissions, user_id={}: {:?}", user_id, e);
            e
        })?;

        tracing::info!("User {} has {} permissions: {:?}", user_id, permissions.len(), permissions);
        Ok(permissions)
    }

    /// Get all minimal menu info for a user by user ID.
    /// Returns a list of AuthMenuInfoEntity for frontend menu tree display.
    /// Optimized version using the helper function from 004_user_info_optimization.sql
    pub async fn get_user_menus(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<AuthMenuInfoEntity>, sqlx::Error> {
        tracing::debug!("Querying menus for user_id: {}", user_id);

        // Use the optimized helper function with proper column mapping
        let menus = sqlx::query_as::<_, AuthMenuInfoEntity>(
            "SELECT id, parent_id, title, path, component, icon, order_num, visible, keep_alive, menu_type FROM get_user_menu_data($1)"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error in get_user_menus, user_id={}: {:?}", user_id, e);
            e
        })?;

        tracing::info!("User {} has {} menus", user_id, menus.len());
        Ok(menus)
    }
}
