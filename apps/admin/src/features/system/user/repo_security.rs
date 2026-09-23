use chrono::Utc;
use rustzen_auth::auth::AuthClaims;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use super::repo::UserRepository;
use crate::{common::error::ServiceError, features::auth::session::SessionRepository};

impl UserRepository {
    pub async fn update_user_password(
        pool: &SqlitePool,
        id: i64,
        password_hash: &str,
        actor: &AuthClaims,
    ) -> Result<bool, ServiceError> {
        let mut tx = pool.begin().await.map_err(|error| {
            tracing::error!(%error, "Failed to begin password reset");
            ServiceError::DatabaseQueryFailed
        })?;
        SessionRepository::assert_actor(
            &mut tx,
            actor,
            Some(rustzen_auth::capability::system_user::RESET_PASSWORD),
            Utc::now().timestamp(),
        )
        .await?;
        Self::assert_owner_for_system_user(&mut tx, actor, id).await?;
        let result = sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
            .bind(password_hash)
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating user password for ID {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        tx.commit().await.map_err(|error| {
            tracing::error!(%error, "Failed to commit password reset");
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_user_status(
        pool: &SqlitePool,
        id: i64,
        status: i16,
        actor: &AuthClaims,
    ) -> Result<bool, ServiceError> {
        let mut tx = pool.begin().await.map_err(|error| {
            tracing::error!(%error, "Failed to begin user status update");
            ServiceError::DatabaseQueryFailed
        })?;
        SessionRepository::assert_actor(
            &mut tx,
            actor,
            Some(rustzen_auth::capability::system_user::UPDATE_STATUS),
            Utc::now().timestamp(),
        )
        .await?;
        Self::assert_owner_for_system_user(&mut tx, actor, id).await?;
        let result = sqlx::query("UPDATE users SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating user status for ID {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        tx.commit().await.map_err(|error| {
            tracing::error!(%error, "Failed to commit user status update");
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(result.rows_affected() > 0)
    }

    pub(super) async fn assert_owner_for_system_user(
        tx: &mut sqlx::Transaction<'_, Sqlite>,
        actor: &AuthClaims,
        user_id: i64,
    ) -> Result<(), ServiceError> {
        let system = sqlx::query_scalar::<_, bool>(
            "SELECT COALESCE((SELECT is_system FROM users WHERE id = ?), FALSE)",
        )
        .bind(user_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(|error| {
            tracing::error!(%error, "Failed to recheck system-user boundary");
            ServiceError::DatabaseQueryFailed
        })?;
        if system {
            SessionRepository::assert_actor(
                tx,
                actor,
                Some(rustzen_auth::capability::SYSTEM_WILDCARD),
                Utc::now().timestamp(),
            )
            .await?;
        }
        Ok(())
    }

    pub(super) async fn assert_owner_for_owner_roles(
        tx: &mut sqlx::Transaction<'_, Sqlite>,
        actor: &AuthClaims,
        role_ids: &[i64],
    ) -> Result<(), ServiceError> {
        if role_ids.is_empty() {
            return Ok(());
        }
        let mut query = QueryBuilder::<Sqlite>::new(
            "SELECT EXISTS(SELECT 1 FROM roles WHERE code = 'owner' AND id IN (",
        );
        let mut separated = query.separated(",");
        for id in role_ids {
            separated.push_bind(id);
        }
        separated.push_unseparated("))");
        let owner_role =
            query.build_query_scalar::<bool>().fetch_one(&mut **tx).await.map_err(|error| {
                tracing::error!(%error, "Failed to recheck owner-role boundary");
                ServiceError::DatabaseQueryFailed
            })?;
        if owner_role {
            SessionRepository::assert_actor(
                tx,
                actor,
                Some(rustzen_auth::capability::SYSTEM_WILDCARD),
                Utc::now().timestamp(),
            )
            .await?;
        }
        Ok(())
    }
}
