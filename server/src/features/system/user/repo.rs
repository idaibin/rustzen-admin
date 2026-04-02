use crate::common::{
    error::ServiceError,
    query::{count_with_filters, fetch_with_filters, push_eq, push_ilike},
};

use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

use super::types::UserWithRolesRow;

/// User db for database operations
pub struct UserRepository;

const DEFAULT_USER_STATUS: i16 = 1;

#[derive(Debug, Clone)]
pub struct UserListQuery {
    pub username: Option<String>,
    pub status: Option<i16>,
    pub real_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub real_name: Option<String>,
    pub status: Option<i16>,
    pub role_ids: Vec<i64>,
}

impl UserRepository {
    fn format_query(query: &UserListQuery, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        push_ilike(query_builder, "username", query.username.as_deref());
        push_ilike(query_builder, "real_name", query.real_name.as_deref());
        push_ilike(query_builder, "email", query.email.as_deref());
        push_eq(query_builder, "status", query.status);
    }

    /// Find users with pagination and filters
    pub async fn list_users(
        pool: &PgPool,
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
        pool: &PgPool,
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
                            .push(" AND (username ILIKE ")
                            .push_bind(pattern.clone())
                            .push(" OR real_name ILIKE ")
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
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<UserWithRolesRow>, ServiceError> {
        sqlx::query_as::<_, UserWithRolesRow>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, is_system, status, last_login_at, created_at, updated_at, roles FROM user_with_roles WHERE id = $1",
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
    pub async fn create_user(pool: &PgPool, cmd: &CreateUserCommand) -> Result<i64, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        let now = Utc::now().naive_utc();

        let user_id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO users (username, email, password_hash, real_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $6)
             RETURNING id",
        )
        .bind(&cmd.username)
        .bind(&cmd.email)
        .bind(&cmd.password_hash)
        .bind(cmd.real_name.as_deref())
        .bind(cmd.status.unwrap_or(DEFAULT_USER_STATUS))
        .bind(now)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error creating user '{}': {:?}", cmd.username, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Self::insert_user_roles(&mut tx, user_id, &cmd.role_ids).await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing user creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(user_id)
    }

    /// Update existing user
    pub async fn update_user(
        pool: &PgPool,
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
             SET email = $1, real_name = $2, updated_at = $4
             WHERE id = $3 AND deleted_at IS NULL
             RETURNING id",
        )
        .bind(email)
        .bind(real_name)
        .bind(id)
        .bind(Utc::now().naive_utc())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

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
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE users SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error soft deleting user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result.rows_affected() > 0)
    }

    /// Set user roles (replace all existing roles)
    pub async fn insert_user_roles(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), ServiceError> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
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
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("INSERT INTO user_roles (user_id, role_id, created_at) ");
        query_builder.push_values(role_ids.iter(), |mut builder, role_id| {
            builder.push_bind(user_id).push_bind(role_id).push_bind(now);
        });

        query_builder.build().execute(&mut **tx).await.map_err(|e| {
            tracing::error!("Database error inserting user_roles: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        Ok(())
    }

    /// Check if email exists
    pub async fn email_exists(pool: &PgPool, email: &str) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND deleted_at IS NULL)",
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

    /// Check if username exists
    pub async fn username_exists(pool: &PgPool, username: &str) -> Result<bool, ServiceError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND deleted_at IS NULL)",
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

    pub async fn update_user_password(
        pool: &PgPool,
        id: i64,
        password_hash: &str,
    ) -> Result<bool, ServiceError> {
        let result = sqlx::query("UPDATE users SET password_hash = $1, updated_at = $3 WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .bind(Utc::now().naive_utc())
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating user password for ID {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn update_user_status(
        pool: &PgPool,
        id: i64,
        status: i16,
    ) -> Result<bool, ServiceError> {
        let result = sqlx::query("UPDATE users SET status = $1, updated_at = $3 WHERE id = $2")
            .bind(status)
            .bind(id)
            .bind(Utc::now().naive_utc())
            .execute(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error updating user status for ID {}: {:?}", id, e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(result.rows_affected() > 0)
    }
}
