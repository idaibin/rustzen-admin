use chrono::Utc;
use rustzen_auth::auth::AuthClaims;
use sqlx::SqlitePool;

use super::repo::{RoleRepository, SoftDeleteOutcome};
use crate::{common::error::ServiceError, features::auth::session::SessionRepository};

impl RoleRepository {
    pub async fn create(
        pool: &SqlitePool,
        role_name: &str,
        role_code: &str,
        description: Option<&str>,
        status: i16,
        menu_ids: &[i64],
        actor: &AuthClaims,
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        SessionRepository::assert_actor(
            &mut tx,
            actor,
            Some(rustzen_auth::capability::system_role::CREATE),
            Utc::now().timestamp(),
        )
        .await?;
        let now = Utc::now().naive_utc();

        let role_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO roles (name, code, description, status, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             RETURNING id",
        )
        .bind(role_name)
        .bind(role_code)
        .bind(description)
        .bind(status)
        .bind(now)
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Self::insert_role_menus(&mut tx, role_id, menu_ids).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing role creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(role_id)
    }

    /// Updates an existing role
    pub async fn update(
        pool: &SqlitePool,
        id: i64,
        request: &super::types::UpdateRolePayload,
        actor: &AuthClaims,
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for role update: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        SessionRepository::assert_actor(
            &mut tx,
            actor,
            Some(rustzen_auth::capability::system_role::UPDATE),
            Utc::now().timestamp(),
        )
        .await?;

        let id_opt = sqlx::query_scalar::<_, i64>(
            "UPDATE roles
                 SET name = ?, code = ?, description = ?, status = ?, updated_at = ?
                 WHERE id = ? AND deleted_at IS NULL
                 RETURNING id",
        )
        .bind(&request.name)
        .bind(&request.code)
        .bind(request.description.as_deref())
        .bind(request.status)
        .bind(Utc::now().naive_utc())
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating role: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        if let Some(id) = id_opt {
            Self::insert_role_menus(&mut tx, id, &request.menu_ids).await?;
            tx.commit().await.map_err(|e| {
                tracing::error!("Database error committing role update transaction: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;
            Ok(id)
        } else {
            Err(ServiceError::NotFound(format!("Role id: {}", id)))
        }
    }

    /// Soft deletes a role
    #[cfg(test)]
    pub async fn soft_delete(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<SoftDeleteOutcome, ServiceError> {
        Self::soft_delete_inner(pool, id, None).await
    }

    pub async fn soft_delete_authorized(
        pool: &SqlitePool,
        id: i64,
        actor: &AuthClaims,
    ) -> Result<SoftDeleteOutcome, ServiceError> {
        Self::soft_delete_inner(pool, id, Some(actor)).await
    }

    async fn soft_delete_inner(
        pool: &SqlitePool,
        id: i64,
        actor: Option<&AuthClaims>,
    ) -> Result<SoftDeleteOutcome, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting role soft-delete transaction {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;
        if let Some(actor) = actor {
            SessionRepository::assert_actor(
                &mut tx,
                actor,
                Some(rustzen_auth::capability::system_role::DELETE),
                Utc::now().timestamp(),
            )
            .await?;
        }
        let result = sqlx::query(
            "UPDATE roles
             SET deleted_at = ?, updated_at = ?
             WHERE id = ?
               AND deleted_at IS NULL
               AND NOT EXISTS (SELECT 1 FROM user_roles WHERE role_id = roles.id)",
        )
        .bind(Utc::now().naive_utc())
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting role {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        let outcome = if result.rows_affected() > 0 {
            SoftDeleteOutcome::Deleted
        } else {
            let (role_is_active, assignment_count): (bool, i64) = sqlx::query_as(
                "SELECT EXISTS(SELECT 1 FROM roles WHERE id = ? AND deleted_at IS NULL),
                        (SELECT COUNT(*) FROM user_roles WHERE role_id = ?)",
            )
            .bind(id)
            .bind(id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error classifying role soft-delete {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

            if role_is_active && assignment_count > 0 {
                SoftDeleteOutcome::AssignmentBlocked(assignment_count)
            } else {
                SoftDeleteOutcome::NotFound
            }
        };

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing role soft-delete {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(outcome)
    }
}
