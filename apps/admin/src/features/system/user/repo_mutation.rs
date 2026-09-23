use chrono::Utc;
use rustzen_auth::auth::AuthClaims;
use sqlx::SqlitePool;

use super::{
    repo::{DEFAULT_USER_STATUS, UserRepository},
    types::CreateUserCommand,
};
use crate::{common::error::ServiceError, features::auth::session::SessionRepository};

impl UserRepository {
    pub async fn create_user(
        pool: &SqlitePool,
        cmd: &CreateUserCommand,
        actor: &AuthClaims,
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        SessionRepository::assert_actor(
            &mut tx,
            actor,
            Some(rustzen_auth::capability::system_user::CREATE),
            Utc::now().timestamp(),
        )
        .await?;
        Self::assert_owner_for_owner_roles(&mut tx, actor, &cmd.role_ids).await?;
        let now = Utc::now().naive_utc();

        let user_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO users (username, email, password_hash, real_name, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(&cmd.username)
        .bind(&cmd.email)
        .bind(&cmd.password_hash)
        .bind(cmd.real_name.as_deref())
        .bind(cmd.status.unwrap_or(DEFAULT_USER_STATUS))
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| Self::map_user_write_error("creating user", e))?;

        Self::insert_user_roles(&mut tx, user_id, &cmd.role_ids).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing user creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(user_id)
    }

    /// Update existing user
    pub async fn update_user(
        pool: &SqlitePool,
        id: i64,
        email: &str,
        real_name: &str,
        role_ids: &[i64],
        actor: &AuthClaims,
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user update: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        SessionRepository::assert_actor(
            &mut tx,
            actor,
            Some(rustzen_auth::capability::system_user::UPDATE),
            Utc::now().timestamp(),
        )
        .await?;
        Self::assert_owner_for_system_user(&mut tx, actor, id).await?;
        Self::assert_owner_for_owner_roles(&mut tx, actor, role_ids).await?;

        let user_id = sqlx::query_scalar::<_, i64>(
            "UPDATE users
             SET email = ?, real_name = ?, updated_at = ?
             WHERE id = ? AND deleted_at IS NULL
             RETURNING id",
        )
        .bind(email)
        .bind(real_name)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| Self::map_user_write_error("updating user", e))?;

        if let Some(id) = user_id {
            Self::insert_user_roles(&mut tx, id, role_ids).await?;
            tx.commit().await.map_err(|e| {
                tracing::error!("Database error committing user update transaction: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
            Ok(id)
        } else {
            Err(ServiceError::NotFound(format!("User id: {}", id)))
        }
    }

    /// Soft delete user
    #[cfg(test)]
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<bool, ServiceError> {
        Self::soft_delete_inner(pool, id, None).await
    }

    pub async fn soft_delete_authorized(
        pool: &SqlitePool,
        id: i64,
        actor: &AuthClaims,
    ) -> Result<bool, ServiceError> {
        Self::soft_delete_inner(pool, id, Some(actor)).await
    }

    async fn soft_delete_inner(
        pool: &SqlitePool,
        id: i64,
        actor: Option<&AuthClaims>,
    ) -> Result<bool, ServiceError> {
        let mut tx = pool.begin().await.map_err(|error| {
            tracing::error!(%error, "Failed to begin user deletion");
            ServiceError::DatabaseQueryFailed
        })?;
        if let Some(actor) = actor {
            SessionRepository::assert_actor(
                &mut tx,
                actor,
                Some(rustzen_auth::capability::system_user::DELETE),
                Utc::now().timestamp(),
            )
            .await?;
            Self::assert_owner_for_system_user(&mut tx, actor, id).await?;
        }
        let result = sqlx::query(
            "UPDATE users SET deleted_at = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            Self::insert_user_roles(&mut tx, id, &[]).await?;
        }
        tx.commit().await.map_err(|error| {
            tracing::error!(%error, "Failed to commit user deletion");
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(deleted)
    }
}
