use std::{collections::HashSet, sync::Arc};

use rustzen_auth::{
    auth::{AuthClaims, CurrentUser},
    capability::SYSTEM_WILDCARD,
};
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::common::error::ServiceError;

const SESSION_CLEANUP_BATCH: i64 = 100;
const MAX_ACTIVE_SESSIONS: i64 = 10;
pub(crate) const DELETE_EXPIRED_SQL: &str = "DELETE FROM access_sessions WHERE sid IN (
         SELECT sid FROM access_sessions WHERE expires_at <= ? ORDER BY expires_at LIMIT ?
     )";
pub(crate) const DELETE_REVOKED_SQL: &str = "DELETE FROM access_sessions WHERE sid IN (
         SELECT sid FROM access_sessions WHERE revoked_at IS NOT NULL ORDER BY revoked_at LIMIT ?
     )";

pub struct SessionRepository;

impl SessionRepository {
    pub async fn assert_actor(
        tx: &mut Transaction<'_, Sqlite>,
        claims: &AuthClaims,
        capability: Option<&str>,
        now: i64,
    ) -> Result<(), ServiceError> {
        if claims.exp as i64 <= now || uuid::Uuid::parse_str(&claims.sid).is_err() {
            return Err(ServiceError::InvalidToken);
        }
        let username = sqlx::query_scalar::<_, String>(
            "SELECT u.username FROM access_sessions s
             INNER JOIN users u ON u.id = s.user_id
             CROSS JOIN access_policy_state p
             WHERE p.id = 1 AND s.sid = ? AND s.user_id = ?
               AND s.revoked_at IS NULL AND s.expires_at > ?
               AND s.auth_epoch_at_issue = ? AND u.auth_epoch = ?
               AND u.status = 1 AND u.deleted_at IS NULL",
        )
        .bind(&claims.sid)
        .bind(claims.user_id)
        .bind(now)
        .bind(claims.user_auth_epoch)
        .bind(claims.user_auth_epoch)
        .fetch_optional(&mut **tx)
        .await
        .map_err(database_error("rechecking access writer"))?
        .filter(|username| username == &claims.username)
        .ok_or(ServiceError::InvalidToken)?;
        if let Some(capability) = capability {
            let permissions = sqlx::query_scalar::<_, String>(
                "SELECT menu_code FROM user_permissions WHERE user_id = ? ORDER BY menu_code",
            )
            .bind(claims.user_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(database_error("rechecking writer permissions"))?;
            let user = CurrentUser::new(claims.user_id, username, permissions, false);
            if !user.has_capability(capability) {
                return Err(ServiceError::PermissionDenied);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    pub async fn create(
        pool: &SqlitePool,
        user_id: i64,
        sid: &str,
        auth_epoch: i64,
        issued_at: i64,
        expires_at: i64,
    ) -> Result<(), ServiceError> {
        let mut tx = pool.begin().await.map_err(database_error("starting session creation"))?;
        Self::create_in_transaction(&mut tx, user_id, sid, auth_epoch, issued_at, expires_at)
            .await?;
        tx.commit().await.map_err(database_error("committing access session"))
    }

    pub(crate) async fn create_in_transaction(
        tx: &mut Transaction<'_, Sqlite>,
        user_id: i64,
        sid: &str,
        auth_epoch: i64,
        issued_at: i64,
        expires_at: i64,
    ) -> Result<(), ServiceError> {
        sqlx::query(DELETE_EXPIRED_SQL)
            .bind(issued_at)
            .bind(SESSION_CLEANUP_BATCH)
            .execute(&mut **tx)
            .await
            .map_err(database_error("cleaning expired sessions"))?;
        sqlx::query(DELETE_REVOKED_SQL)
            .bind(SESSION_CLEANUP_BATCH)
            .execute(&mut **tx)
            .await
            .map_err(database_error("cleaning revoked sessions"))?;

        let inserted = sqlx::query(
            "INSERT INTO access_sessions
                 (sid, user_id, auth_epoch_at_issue, expires_at, created_at)
             SELECT ?, id, auth_epoch, ?, ? FROM users
             WHERE id = ? AND status = 1 AND deleted_at IS NULL AND auth_epoch = ?",
        )
        .bind(sid)
        .bind(expires_at)
        .bind(issued_at)
        .bind(user_id)
        .bind(auth_epoch)
        .execute(&mut **tx)
        .await
        .map_err(database_error("creating access session"))?;
        if inserted.rows_affected() != 1 {
            return Err(ServiceError::InvalidToken);
        }

        sqlx::query(
            "UPDATE access_sessions SET revoked_at = ?
             WHERE user_id = ? AND revoked_at IS NULL AND expires_at > ?
               AND rowid NOT IN (
                   SELECT rowid FROM access_sessions
                   WHERE user_id = ? AND revoked_at IS NULL AND expires_at > ?
                   ORDER BY rowid DESC LIMIT ?
               )",
        )
        .bind(issued_at)
        .bind(user_id)
        .bind(issued_at)
        .bind(user_id)
        .bind(issued_at)
        .bind(MAX_ACTIVE_SESSIONS)
        .execute(&mut **tx)
        .await
        .map_err(database_error("enforcing active session limit"))?;
        Ok(())
    }

    pub async fn load_authoritative_user(
        pool: &SqlitePool,
        claims: &AuthClaims,
        now: i64,
    ) -> Result<CurrentUser, ServiceError> {
        if claims.exp as i64 <= now
            || claims.user_auth_epoch < 1
            || uuid::Uuid::parse_str(&claims.sid).is_err()
        {
            return Err(ServiceError::InvalidToken);
        }
        let mut tx = pool.begin().await.map_err(database_error("starting access snapshot"))?;
        let row = sqlx::query_as::<_, (String, i64)>(
            "SELECT u.username, p.authz_epoch
             FROM access_sessions s
             INNER JOIN users u ON u.id = s.user_id
             CROSS JOIN access_policy_state p
             WHERE p.id = 1 AND s.sid = ? AND s.user_id = ?
               AND s.revoked_at IS NULL AND s.expires_at > ?
               AND s.auth_epoch_at_issue = ? AND u.auth_epoch = ?
               AND u.status = 1 AND u.deleted_at IS NULL",
        )
        .bind(&claims.sid)
        .bind(claims.user_id)
        .bind(now)
        .bind(claims.user_auth_epoch)
        .bind(claims.user_auth_epoch)
        .fetch_optional(&mut *tx)
        .await
        .map_err(database_error("loading authoritative access session"))?
        .filter(|(username, _)| username == &claims.username)
        .ok_or(ServiceError::InvalidToken)?;

        let permissions = sqlx::query_scalar::<_, String>(
            "SELECT menu_code FROM user_permissions WHERE user_id = ? ORDER BY menu_code",
        )
        .bind(claims.user_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(database_error("loading authoritative permissions"))?;
        tx.commit().await.map_err(database_error("closing access snapshot"))?;
        let permissions = Arc::new(permissions.into_iter().collect::<HashSet<_>>());
        Ok(CurrentUser {
            user_id: claims.user_id,
            username: row.0,
            is_super: permissions.contains(SYSTEM_WILDCARD),
            permissions,
        })
    }

    pub async fn revoke_sid(
        pool: &SqlitePool,
        user_id: i64,
        sid: &str,
        now: i64,
    ) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE access_sessions SET revoked_at = COALESCE(revoked_at, ?)
             WHERE sid = ? AND user_id = ?",
        )
        .bind(now)
        .bind(sid)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(database_error("revoking access session"))?;
        Ok(())
    }

    pub async fn revoke_all(
        pool: &SqlitePool,
        user_id: i64,
        now: i64,
    ) -> Result<bool, ServiceError> {
        let mut tx = pool.begin().await.map_err(database_error("starting session revocation"))?;
        let updated = sqlx::query(
            "UPDATE users SET auth_epoch = auth_epoch + 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(database_error("advancing user auth epoch"))?;
        if updated.rows_affected() == 1 {
            sqlx::query(
                "UPDATE access_sessions SET revoked_at = COALESCE(revoked_at, ?)
                 WHERE user_id = ?",
            )
            .bind(now)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(database_error("revoking user sessions"))?;
        }
        tx.commit().await.map_err(database_error("committing session revocation"))?;
        Ok(updated.rows_affected() == 1)
    }

    pub async fn revoke_all_authorized(
        pool: &SqlitePool,
        user_id: i64,
        actor: &AuthClaims,
        capability: &str,
        now: i64,
    ) -> Result<bool, ServiceError> {
        let mut tx = pool.begin().await.map_err(database_error("starting session revocation"))?;
        Self::assert_actor(&mut tx, actor, Some(capability), now).await?;
        let system_user = sqlx::query_scalar::<_, bool>(
            "SELECT COALESCE((SELECT is_system FROM users WHERE id = ?), FALSE)",
        )
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(database_error("checking session-revoke target"))?;
        if system_user {
            Self::assert_actor(
                &mut tx,
                actor,
                Some(rustzen_auth::capability::SYSTEM_WILDCARD),
                now,
            )
            .await?;
        }
        let updated = sqlx::query(
            "UPDATE users SET auth_epoch = auth_epoch + 1, updated_at = CURRENT_TIMESTAMP
             WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(database_error("advancing user auth epoch"))?;
        if updated.rows_affected() == 1 {
            sqlx::query(
                "UPDATE access_sessions SET revoked_at = COALESCE(revoked_at, ?)
                 WHERE user_id = ?",
            )
            .bind(now)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(database_error("revoking user sessions"))?;
        }
        tx.commit().await.map_err(database_error("committing session revocation"))?;
        Ok(updated.rows_affected() == 1)
    }
}

fn database_error(context: &'static str) -> impl FnOnce(sqlx::Error) -> ServiceError {
    move |error| {
        tracing::error!(%error, context, "Access session database operation failed");
        ServiceError::DatabaseQueryFailed
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
