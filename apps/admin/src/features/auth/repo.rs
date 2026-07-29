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
            "SELECT id, password_hash, status FROM users WHERE username = ? AND deleted_at IS NULL",
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
        let now = Utc::now().naive_utc();
        sqlx::query("UPDATE users SET last_login_at = ?, updated_at = ? WHERE id = ?")
            .bind(now)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_last_login, user_id={}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::AuthRepository;

    #[tokio::test]
    async fn update_last_login_sets_both_timestamps_for_the_requested_user() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("connect");
        crate::infra::db::run_migrations(&pool).await.expect("migrations");

        AuthRepository::update_last_login(&pool, 1).await.expect("update last login");

        let (last_login_at, updated_at): (Option<String>, String) =
            sqlx::query_as("SELECT last_login_at, updated_at FROM users WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load timestamps");
        assert!(last_login_at.is_some());
        assert_eq!(last_login_at.as_deref(), Some(updated_at.as_str()));
    }
}
