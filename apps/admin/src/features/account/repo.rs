use super::types::{PasswordHashRow, UpdateAccountProfileRequest};
use crate::{common::error::ServiceError, features::auth::session::SessionRepository};

use chrono::Utc;
use rustzen_auth::auth::AuthClaims;
use sqlx::SqlitePool;

/// Current-account db operations.
pub struct AccountRepository;

impl AccountRepository {
    #[cfg(feature = "full")]
    pub async fn update_avatar(
        pool: &SqlitePool,
        user_id: i64,
        avatar_url: &str,
        actor: &AuthClaims,
    ) -> Result<(), ServiceError> {
        let mut tx = pool.begin().await.map_err(database_error("starting avatar update"))?;
        SessionRepository::assert_actor(&mut tx, actor, None, Utc::now().timestamp()).await?;
        sqlx::query("UPDATE users SET avatar_url = ?, updated_at = ? WHERE id = ?")
            .bind(avatar_url)
            .bind(Utc::now().naive_utc())
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_avatar, user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        tx.commit().await.map_err(database_error("committing avatar update"))
    }

    pub async fn email_exists_for_other_user(
        pool: &SqlitePool,
        user_id: i64,
        email: &str,
    ) -> Result<bool, ServiceError> {
        sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = ? AND id <> ? AND deleted_at IS NULL)",
        )
        .bind(email)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error in email_exists_for_other_user, user_id={}: {:?}",
                user_id,
                e
            );
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn update_profile(
        pool: &SqlitePool,
        user_id: i64,
        request: &UpdateAccountProfileRequest,
        actor: &AuthClaims,
    ) -> Result<(), ServiceError> {
        let mut tx = pool.begin().await.map_err(database_error("starting profile update"))?;
        SessionRepository::assert_actor(&mut tx, actor, None, Utc::now().timestamp()).await?;
        sqlx::query("UPDATE users SET email = ?, real_name = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(&request.email)
            .bind(&request.real_name)
            .bind(Utc::now().naive_utc())
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error in update_profile, user_id={}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        tx.commit().await.map_err(database_error("committing profile update"))
    }

    pub async fn find_password_hash_by_id(
        pool: &SqlitePool,
        user_id: i64,
    ) -> Result<Option<PasswordHashRow>, ServiceError> {
        sqlx::query_as::<_, PasswordHashRow>(
            "SELECT password_hash FROM users WHERE id = ? AND deleted_at IS NULL",
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
        pool: &SqlitePool,
        user_id: i64,
        expected_password_hash: &str,
        new_password_hash: &str,
    ) -> Result<bool, ServiceError> {
        Self::update_password_inner(pool, user_id, expected_password_hash, new_password_hash, None)
            .await
    }

    pub async fn update_password_authorized(
        pool: &SqlitePool,
        user_id: i64,
        expected_password_hash: &str,
        new_password_hash: &str,
        actor: &AuthClaims,
    ) -> Result<bool, ServiceError> {
        Self::update_password_inner(
            pool,
            user_id,
            expected_password_hash,
            new_password_hash,
            Some(actor),
        )
        .await
    }

    async fn update_password_inner(
        pool: &SqlitePool,
        user_id: i64,
        expected_password_hash: &str,
        new_password_hash: &str,
        actor: Option<&AuthClaims>,
    ) -> Result<bool, ServiceError> {
        let mut tx = pool.begin().await.map_err(database_error("starting password change"))?;
        if let Some(actor) = actor {
            SessionRepository::assert_actor(&mut tx, actor, None, Utc::now().timestamp()).await?;
        }
        let result = sqlx::query(
            "UPDATE users SET password_hash = ?, updated_at = ?
             WHERE id = ? AND deleted_at IS NULL AND password_hash = ?",
        )
        .bind(new_password_hash)
        .bind(Utc::now().naive_utc())
        .bind(user_id)
        .bind(expected_password_hash)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error in update_password, user_id={}: {:?}", user_id, e);
            ServiceError::DatabaseQueryFailed
        })?;
        tx.commit().await.map_err(database_error("committing password change"))?;
        Ok(result.rows_affected() > 0)
    }
}

fn database_error(context: &'static str) -> impl FnOnce(sqlx::Error) -> ServiceError {
    move |error| {
        tracing::error!(%error, context, "Account database operation failed");
        ServiceError::DatabaseQueryFailed
    }
}
