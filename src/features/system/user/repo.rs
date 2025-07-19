// backend/src/features/system/user/repo.rs

// This module implements all database operations related to the `users` table.
//
// Functions in this module are designed to be simple, direct database
// interactions. All logic, such as mapping to response models, checking
// permissions, or combining multiple operations, should be handled in the
// service layer.

use super::{
    dto::CreateUserDto,
    entity::{UserEntity, UserWithRolesEntity},
};
use crate::{common::error::ServiceError, features::system::user::dto::UserQueryDto};
use chrono::Utc;
use sqlx::{PgPool, QueryBuilder};

/// User repository for database operations
pub struct UserRepository;

impl UserRepository {
    fn format_query(query: &UserQueryDto, query_builder: &mut QueryBuilder<'_, sqlx::Postgres>) {
        if let Some(username) = &query.username {
            if !username.trim().is_empty() {
                query_builder.push(" AND username ILIKE ").push_bind(format!("%{}%", username));
            }
        }
        if let Some(real_name) = &query.real_name {
            if !real_name.trim().is_empty() {
                query_builder.push(" AND real_name ILIKE ").push_bind(format!("%{}%", real_name));
            }
        }
        if let Some(email) = &query.email {
            if !email.trim().is_empty() {
                query_builder.push(" AND email ILIKE ").push_bind(format!("%{}%", email));
            }
        }
        if let Some(status) = &query.status {
            if let Ok(status_num) = status.parse::<i16>() {
                query_builder.push(" AND status = ").push_bind(status_num);
            }
        }
    }

    /// Count users matching filters
    async fn count_users(pool: &PgPool, query: &UserQueryDto) -> Result<i64, ServiceError> {
        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM user_with_roles WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        let count: (i64,) = query_builder.build_query_as().fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;
        tracing::info!("user count: {:?}", count);

        Ok(count.0)
    }

    /// Find users with pagination and filters
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        query: UserQueryDto,
    ) -> Result<(Vec<UserWithRolesEntity>, i64), ServiceError> {
        tracing::debug!("Finding users with pagination and filters: {:?}", query);
        let total = Self::count_users(pool, &query).await?;
        if total == 0 {
            return Ok((Vec::new(), total));
        }

        let mut query_builder: QueryBuilder<'_, sqlx::Postgres> =
            QueryBuilder::new("SELECT * FROM user_with_roles WHERE 1=1");

        Self::format_query(&query, &mut query_builder);

        query_builder.push(" ORDER BY created_at DESC");
        query_builder.push(" LIMIT ").push_bind(limit);
        query_builder.push(" OFFSET ").push_bind(offset);

        let users = query_builder.build_query_as().fetch_all(pool).await.map_err(|e| {
            tracing::error!("Database error in user_with_roles pagination: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok((users, total))
    }

    /// Find users for dropdown options
    pub async fn find_options(
        pool: &PgPool,
        status: Option<i16>, // 1, 2, or None (all users)
        q: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, ServiceError> {
        let mut query = String::from(
            "SELECT id, COALESCE(real_name, username) as display_name
             FROM users
             WHERE deleted_at IS NULL",
        );

        // Handle status filter
        if let Some(status_val) = status {
            query.push_str(&format!(" AND status = {}", status_val));
        }

        // Handle search query
        if let Some(search_term) = q {
            if !search_term.trim().is_empty() {
                query.push_str(&format!(
                    " AND (username ILIKE '%{}%' OR real_name ILIKE '%{}%')",
                    search_term.replace("'", "''"),
                    search_term.replace("'", "''")
                ));
            }
        }

        query.push_str(" ORDER BY display_name");

        // Handle limit
        if let Some(limit_val) = limit {
            query.push_str(&format!(" LIMIT {}", limit_val));
        }

        let result =
            sqlx::query_as::<_, (i64, String)>(&query).fetch_all(pool).await.map_err(|e| {
                tracing::error!("Database error finding user options: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(result)
    }

    /// Find user by ID
    pub async fn find_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<Option<UserWithRolesEntity>, ServiceError> {
        let result = sqlx::query_as::<_, UserWithRolesEntity>(
            "SELECT *
             FROM user_with_roles
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user by ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result)
    }

    /// Create new user with optional roles (unified method)
    pub async fn create_user(
        pool: &PgPool,
        dto: &CreateUserDto,
    ) -> Result<UserEntity, ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for user creation: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Create user
        let user = match sqlx::query_as::<_, UserEntity>(
            "INSERT INTO users (username, email, password_hash, real_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $6)
             RETURNING id, username, email, password_hash, real_name, avatar_url, status"
        )
        .bind(&dto.username)
        .bind(&dto.email)
        .bind(&dto.password)
        .bind(dto.real_name.as_deref())
        .bind(dto.status.unwrap_or(1))
        .bind(Utc::now().naive_utc())
        .fetch_one(&mut *tx)
        .await
        {
            Ok(user) => user,
            Err(e) => {
                tracing::error!("Database error creating user '{}': {:?}", dto.username, e);
                let _ = tx.rollback().await;
                return Err(ServiceError::DatabaseQueryFailed);
            }
        };

        // Set user roles if provided
        if !dto.role_ids.is_empty() {
            // Validate all role IDs exist
            let valid_roles = match sqlx::query_as::<_, (i64,)>(
                "SELECT id FROM roles WHERE id = ANY($1) AND deleted_at IS NULL AND status = 1",
            )
            .bind(&dto.role_ids)
            .fetch_all(&mut *tx)
            .await
            {
                Ok(roles) => roles,
                Err(e) => {
                    tracing::error!("Database error validating role IDs: {:?}", e);
                    let _ = tx.rollback().await;
                    return Err(ServiceError::DatabaseQueryFailed);
                }
            };

            if valid_roles.len() != dto.role_ids.len() {
                // Invalid role IDs, rollback transaction
                tracing::error!("Invalid role IDs provided for user creation: {:?}", dto.role_ids);
                let _ = tx.rollback().await;
                return Err(ServiceError::InvalidRoleId);
            }

            // Insert user role associations
            let mut query_builder =
                "INSERT INTO user_roles (user_id, role_id, created_at) VALUES ".to_string();
            let now = Utc::now().naive_utc();
            for (i, role_id) in dto.role_ids.iter().enumerate() {
                if i > 0 {
                    query_builder.push_str(", ");
                }
                query_builder.push_str(&format!("({}, {}, '{}')", user.id, role_id, now));
            }

            if let Err(e) = sqlx::query(&query_builder).execute(&mut *tx).await {
                tracing::error!("Database error inserting user roles: {:?}", e);
                let _ = tx.rollback().await;
                return Err(ServiceError::DatabaseQueryFailed);
            }
        }

        // Commit transaction
        tx.commit().await.map_err(|e| {
            tracing::error!("Database error committing user creation transaction: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(user)
    }

    /// Update existing user
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        email: Option<String>,
        real_name: Option<String>,
        status: Option<i16>,
        password_hash: Option<String>,
    ) -> Result<bool, ServiceError> {
        let result = sqlx::query(
            "UPDATE users
             SET email = COALESCE($1, email),
                 real_name = COALESCE($2, real_name),
                 status = COALESCE($3, status),
                 password_hash = COALESCE($4, password_hash),
                 updated_at = NOW()
             WHERE id = $5 AND deleted_at IS NULL",
        )
        .bind(email)
        .bind(real_name)
        .bind(status)
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result.rows_affected() > 0)
    }

    /// Soft delete user
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, ServiceError> {
        let result =
            sqlx::query("UPDATE users SET deleted_at = $1 WHERE id = $2 AND deleted_at IS NULL")
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
    pub async fn set_user_roles(
        pool: &PgPool,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), ServiceError> {
        let mut tx = pool.begin().await.map_err(|e| {
            tracing::error!("Database error starting transaction for setting user roles: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Delete existing user roles
        if let Err(e) = sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await
        {
            tracing::error!(
                "Database error deleting existing user roles for user ID {}: {:?}",
                user_id,
                e
            );
            let _ = tx.rollback().await;
            return Err(ServiceError::DatabaseQueryFailed);
        }

        // Insert new user roles if any
        if !role_ids.is_empty() {
            // Validate all role IDs exist
            let valid_roles = match sqlx::query_as::<_, (i64,)>(
                "SELECT id FROM roles WHERE id = ANY($1) AND deleted_at IS NULL AND status = 1",
            )
            .bind(role_ids)
            .fetch_all(&mut *tx)
            .await
            {
                Ok(roles) => roles,
                Err(e) => {
                    tracing::error!(
                        "Database error validating role IDs for user {}: {:?}",
                        user_id,
                        e
                    );
                    let _ = tx.rollback().await;
                    return Err(ServiceError::DatabaseQueryFailed);
                }
            };

            if valid_roles.len() != role_ids.len() {
                tracing::error!("Invalid role IDs provided for user {}: {:?}", user_id, role_ids);
                let _ = tx.rollback().await;
                return Err(ServiceError::InvalidRoleId);
            }

            // Insert user role associations
            let mut query_builder =
                "INSERT INTO user_roles (user_id, role_id, created_at) VALUES ".to_string();
            let now = Utc::now().naive_utc();
            for (i, role_id) in role_ids.iter().enumerate() {
                if i > 0 {
                    query_builder.push_str(", ");
                }
                query_builder.push_str(&format!("({}, {}, '{}')", user_id, role_id, now));
            }

            if let Err(e) = sqlx::query(&query_builder).execute(&mut *tx).await {
                tracing::error!(
                    "Database error inserting user roles for user {}: {:?}",
                    user_id,
                    e
                );
                let _ = tx.rollback().await;
                return Err(ServiceError::DatabaseQueryFailed);
            }
        }

        tx.commit().await.map_err(|e| {
            tracing::error!(
                "Database error committing user roles transaction for user {}: {:?}",
                user_id,
                e
            );
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
}
