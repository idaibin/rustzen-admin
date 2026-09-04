use crate::common::{
    error::ServiceError,
    query::{count_with_filters, fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{Error as SqlxError, QueryBuilder, Sqlite, SqlitePool};

#[cfg(feature = "full")]
use super::types::UserDashboardCounts;
use super::types::{CreateUserCommand, UserListQuery, UserWithRolesRow};

/// User db for database operations
pub struct UserRepository;

const DEFAULT_USER_STATUS: i16 = 1;

impl UserRepository {
    fn format_query(query: &UserListQuery, query_builder: &mut QueryBuilder<Sqlite>) {
        push_ilike(query_builder, "username", query.username.as_deref());
        push_ilike(query_builder, "real_name", query.real_name.as_deref());
        push_ilike(query_builder, "email", query.email.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Find users with pagination and filters
    pub async fn list_users(
        pool: &SqlitePool,
        offset: i64,
        limit: i64,
        query: UserListQuery,
    ) -> Result<(Vec<UserWithRolesRow>, i64), ServiceError> {
        tracing::debug!("Finding users with pagination and filters: {:?}", query);
        let total = count_with_filters(
            pool,
            "SELECT COUNT(*) FROM user_with_roles WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
        )
        .await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }
        let users = fetch_with_filters(
            pool,
            "SELECT id, username, email, password_hash, real_name, avatar_url, is_system, status, last_login_at, created_at, updated_at, roles FROM user_with_roles WHERE 1=1",
            |query_builder| {
                Self::format_query(&query, query_builder);
            },
            Some("created_at DESC"),
            Some(limit),
            Some(offset),
        )
        .await?;

        Ok((users, total))
    }

    /// Find users for dropdown options
    pub async fn list_user_options(
        pool: &SqlitePool,
        status: Option<i16>,
        q: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        fetch_with_filters(
            pool,
            "SELECT id, COALESCE(real_name, username) AS label FROM users WHERE deleted_at IS NULL",
            |query_builder| {
                push_eq(query_builder, "status", status);
                if let Some(search_term) = q {
                    let search_term = search_term.trim();
                    if !search_term.is_empty() {
                        let pattern = format!("%{}%", search_term);
                        query_builder
                            .push(" AND (LOWER(username) LIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR LOWER(real_name) LIKE ")
                            .push_bind(pattern)
                            .push(")");
                    }
                }
            },
            Some("label ASC"),
            limit,
            None,
        )
        .await
    }

    /// Find user by ID (returns None if not found)
    pub async fn find_user_by_id(
        pool: &SqlitePool,
        id: i64,
    ) -> Result<Option<UserWithRolesRow>, ServiceError> {
        sqlx::query_as::<_, UserWithRolesRow>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, is_system, status, last_login_at, created_at, updated_at, roles FROM user_with_roles WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user by ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })
    }

    /// Create new user with optional roles (unified method)
    pub async fn create_user(
        pool: &SqlitePool,
        cmd: &CreateUserCommand,
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
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
    ) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user update: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

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
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<bool, ServiceError> {
        let mut tx = pool.begin().await.map_err(|error| {
            tracing::error!(%error, "Failed to begin user deletion");
            ServiceError::DatabaseQueryFailed
        })?;
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

    /// Set user roles (replace all existing roles)
    pub async fn insert_user_roles(
        tx: &mut sqlx::Transaction<'_, Sqlite>,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!("Database error deleting existing user_roles: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        if role_ids.is_empty() {
            return Ok(());
        }
        let now = Utc::now().naive_utc();
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "INSERT INTO user_roles (user_id, role_id, created_at)
             SELECT ",
        );
        query_builder.push_bind(user_id).push(", requested.role_id, ").push_bind(now);
        query_builder.push(" FROM (SELECT ");
        query_builder.push_bind(role_ids[0]).push(" AS role_id");
        for role_id in &role_ids[1..] {
            query_builder.push(" UNION ALL SELECT ").push_bind(role_id);
        }
        query_builder.push(
            ") AS requested
             INNER JOIN roles r ON r.id = requested.role_id AND r.deleted_at IS NULL",
        );

        let result = query_builder.build().execute(&mut **tx).await.map_err(|e| {
            tracing::error!("Database error inserting user_roles: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        if result.rows_affected() != role_ids.len() as u64 {
            return Err(ServiceError::NotFound("Role".to_string()));
        }
        Ok(())
    }

    /// Check if email exists
    pub async fn email_exists(pool: &SqlitePool, email: &str) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = ? AND deleted_at IS NULL)",
        )
        .bind(email)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking email existence '{}': {:?}", email, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(exists)
    }

    pub async fn email_exists_for_other_user(
        pool: &SqlitePool,
        email: &str,
        user_id: i64,
    ) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = ? AND id != ? AND deleted_at IS NULL)",
        )
        .bind(email)
        .bind(user_id)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Database error checking email existence '{}' excluding user {}: {:?}",
                email,
                user_id,
                e
            );
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(exists)
    }

    /// Check if username exists
    pub async fn username_exists(pool: &SqlitePool, username: &str) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ? AND deleted_at IS NULL)",
        )
        .bind(username)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking username existence '{}': {:?}", username, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(exists)
    }

    pub async fn list_role_identities_by_ids(
        pool: &SqlitePool,
        role_ids: &[i64],
    ) -> Result<Vec<(i64, String, bool)>, ServiceError> {
        if role_ids.is_empty() {
            return Ok(Vec::new());
        }

        let unique_role_ids = role_ids.iter().copied().collect::<std::collections::HashSet<_>>();
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, code, is_system FROM roles WHERE deleted_at IS NULL AND id IN (",
        );
        let mut separated = query_builder.separated(", ");
        for role_id in unique_role_ids {
            separated.push_bind(role_id);
        }
        separated.push_unseparated(")");

        query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error listing role identities by ids: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })
    }

    pub async fn update_user_password(
        pool: &SqlitePool,
        id: i64,
        password_hash: &str,
    ) -> Result<bool, ServiceError> {
        let result = sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
            .bind(password_hash)
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating user password for ID {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_user_status(
        pool: &SqlitePool,
        id: i64,
        status: i16,
    ) -> Result<bool, ServiceError> {
        let result = sqlx::query("UPDATE users SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status)
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating user status for ID {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(result.rows_affected() > 0)
    }

    #[cfg(feature = "full")]
    pub async fn dashboard_counts(pool: &SqlitePool) -> Result<UserDashboardCounts, ServiceError> {
        let (total_users, active_users, today_logins, pending_users) = tokio::join!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
                .fetch_one(pool),
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE status = 1 AND last_login_at > datetime('now', '-7 day') AND deleted_at IS NULL",
            )
            .fetch_one(pool),
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE last_login_at > datetime('now', '-1 day') AND deleted_at IS NULL",
            )
            .fetch_one(pool),
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE status = 3 AND deleted_at IS NULL",
            )
            .fetch_one(pool),
        );

        Ok(UserDashboardCounts {
            total_users: map_dashboard_count_result(total_users, "total users")?,
            active_users: map_dashboard_count_result(active_users, "active users")?,
            today_logins: map_dashboard_count_result(today_logins, "today logins")?,
            pending_users: map_dashboard_count_result(pending_users, "pending users")?,
        })
    }

    fn map_user_write_error(context: &str, err: SqlxError) -> ServiceError {
        if let SqlxError::Database(db_err) = &err
            && let Some(conflict) = classify_user_unique_conflict(
                db_err.code().as_deref(),
                db_err.constraint(),
                db_err.message(),
            )
        {
            match conflict {
                UserUniqueConflict::Username => {
                    tracing::warn!("Unique username conflict while {}", context);
                    return ServiceError::UsernameConflict;
                }
                UserUniqueConflict::Email => {
                    tracing::warn!("Unique email conflict while {}", context);
                    return ServiceError::EmailConflict;
                }
            }
        }

        tracing::error!("Database error {}: {:?}", context, err);
        ServiceError::DatabaseQueryFailed
    }
}

#[cfg(feature = "full")]
fn map_dashboard_count_result(
    result: Result<i64, SqlxError>,
    metric_name: &str,
) -> Result<i64, ServiceError> {
    result.map_err(|e| {
        tracing::error!("Database error getting dashboard {} count: {:?}", metric_name, e);
        ServiceError::DatabaseQueryFailed
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UserUniqueConflict {
    Username,
    Email,
}

fn classify_user_unique_conflict(
    code: Option<&str>,
    constraint: Option<&str>,
    message: &str,
) -> Option<UserUniqueConflict> {
    match constraint {
        Some("idx_users_username") => {
            return Some(UserUniqueConflict::Username);
        }
        Some("idx_users_email") => return Some(UserUniqueConflict::Email),
        _ => {}
    }

    if !matches!(code, Some("2067" | "1555")) {
        return None;
    }

    if message.contains("users.username") || message.contains("idx_users_username") {
        return Some(UserUniqueConflict::Username);
    }
    if message.contains("users.email") || message.contains("idx_users_email") {
        return Some(UserUniqueConflict::Email);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::{UserUniqueConflict, classify_user_unique_conflict};

    #[tokio::test]
    async fn deleting_user_releases_roles_atomically() {
        use super::UserRepository;
        use crate::features::system::role::repo::{RoleRepository, SoftDeleteOutcome};
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::infra::db::run_migrations(&pool).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys").fetch_one(&pool).await.unwrap(),
            1
        );
        let role: i64 = sqlx::query_scalar("INSERT INTO roles (name, code, status, is_system) VALUES ('Temporary', 'delete_test', 1, FALSE) RETURNING id")
            .fetch_one(&pool).await.unwrap();
        let user: i64 = sqlx::query_scalar("INSERT INTO users (username, email, password_hash, status, is_system) VALUES ('delete_test', 'delete@test.local', 'unused', 1, FALSE) RETURNING id")
            .fetch_one(&pool).await.unwrap();
        let mut tx = pool.begin().await.unwrap();
        UserRepository::insert_user_roles(&mut tx, user, &[role]).await.unwrap();
        tx.commit().await.unwrap();
        sqlx::query("CREATE TRIGGER fail_role_cleanup BEFORE DELETE ON user_roles BEGIN SELECT RAISE(ABORT, 'test cleanup failure'); END")
            .execute(&pool).await.unwrap();
        assert!(UserRepository::soft_delete(&pool, user).await.is_err());
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE id = ? AND deleted_at IS NULL"
            )
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap(),
            1
        );
        assert_eq!(RoleRepository::get_role_user_count(&pool, role).await.unwrap(), 1);
        sqlx::query("DROP TRIGGER fail_role_cleanup").execute(&pool).await.unwrap();
        assert!(UserRepository::soft_delete(&pool, user).await.unwrap());
        assert!(!UserRepository::soft_delete(&pool, user).await.unwrap());
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM users WHERE id = ? AND deleted_at IS NOT NULL"
            )
            .bind(user)
            .fetch_one(&pool)
            .await
            .unwrap(),
            1
        );
        assert_eq!(RoleRepository::get_role_user_count(&pool, role).await.unwrap(), 0);
        assert_eq!(
            RoleRepository::soft_delete(&pool, role).await.unwrap(),
            SoftDeleteOutcome::Deleted
        );
        pool.close().await;
    }

    #[test]
    fn classify_user_unique_conflict_returns_username_for_sqlite_username_message() {
        let conflict = classify_user_unique_conflict(
            Some("2067"),
            None,
            "UNIQUE constraint failed: users.username",
        );

        assert_eq!(conflict, Some(UserUniqueConflict::Username));
    }

    #[test]
    fn classify_user_unique_conflict_returns_email_for_sqlite_email_message() {
        let conflict = classify_user_unique_conflict(
            Some("2067"),
            None,
            "UNIQUE constraint failed: users.email",
        );

        assert_eq!(conflict, Some(UserUniqueConflict::Email));
    }

    #[test]
    fn classify_user_unique_conflict_ignores_unrelated_sqlite_constraint() {
        let conflict = classify_user_unique_conflict(
            Some("2067"),
            None,
            "UNIQUE constraint failed: roles.code",
        );

        assert_eq!(conflict, None);
    }
}
