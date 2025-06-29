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
        let user = sqlx::query_as::<_, LoginCredentialsEntity>(
            "SELECT id, username, password_hash, status FROM users WHERE username = $1 AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    /// Find user by ID for authentication (returns AuthUserInfo)
    pub async fn get_user_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<AuthUserInfo>, sqlx::Error> {
        let user = sqlx::query_as::<_, AuthUserInfo>(
            "SELECT id, username, real_name, avatar_url FROM users WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(user)
    }

    /// Update last login timestamp
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get all permission keys for a user by user ID.
    /// Returns a list of permission strings (e.g., "system:user:list").
    pub async fn get_user_permissions(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        // Query all permission keys for the user via role-menu-permission relationship
        let permissions = sqlx::query_scalar::<_, String>(
            r#"
            SELECT DISTINCT m.permission
            FROM user_roles ur
            JOIN role_menus rm ON ur.role_id = rm.role_id
            JOIN menus m ON rm.menu_id = m.id
            WHERE ur.user_id = $1 AND m.permission IS NOT NULL AND m.permission != ''
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(permissions)
    }

    /// Get all minimal menu info for a user by user ID.
    /// Returns a list of AuthMenuInfoEntity for frontend menu tree display.
    pub async fn get_user_menus(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<AuthMenuInfoEntity>, sqlx::Error> {
        // Query all menus for the user via role-menu relationship
        let menus = sqlx::query_as::<_, AuthMenuInfoEntity>(
            r#"
            SELECT DISTINCT m.id, m.parent_id, m.title, m.path, m.component, m.icon, m.order_num, m.visible, m.keep_alive, m.type as menu_type
            FROM user_roles ur
            JOIN role_menus rm ON ur.role_id = rm.role_id
            JOIN menus m ON rm.menu_id = m.id
            WHERE ur.user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;
        Ok(menus)
    }
}
