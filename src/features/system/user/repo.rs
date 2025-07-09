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
use crate::common::error::ServiceError;
use chrono::Utc;
use sqlx::PgPool;

/// User repository for database operations
pub struct UserRepository;

impl UserRepository {
    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<UserEntity>, ServiceError> {
        let result = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at
             FROM users
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

    /// Find user by username
    pub async fn find_by_username(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<UserEntity>, ServiceError> {
        let result = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at
             FROM users
             WHERE username = $1 AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user by username '{}': {:?}", username, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result)
    }

    /// Find user by email
    pub async fn find_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<UserEntity>, ServiceError> {
        let result = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at
             FROM users
             WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error finding user by email '{}': {:?}", email, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result)
    }

    /// Find users with pagination and filters
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        username_filter: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<Vec<UserWithRolesEntity>, ServiceError> {
        let sql = r#"
            SELECT *
            FROM user_with_roles
            WHERE
                ($1::text IS NULL OR username ILIKE '%' || $1 || '%' OR real_name ILIKE '%' || $1 || '%')
                AND ($2::text IS NULL OR status::text = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
        "#;

        let users: Vec<UserWithRolesEntity> = sqlx::query_as(sql)
            .bind(username_filter)
            .bind(status_filter)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error in user_with_roles pagination: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(users)
    }

    /// Count users matching filters
    pub async fn count_users(
        pool: &PgPool,
        username_filter: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<i64, ServiceError> {
        let mut query = String::from("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL");

        // Handle status filter
        if let Some(status_str) = status_filter {
            match status_str {
                "1" => query.push_str(" AND status = 1"),
                "2" => query.push_str(" AND status = 2"),
                "all" => {}                             // No status filter
                _ => query.push_str(" AND status = 1"), // Default to active users
            }
        }

        // Handle username search
        if let Some(keyword) = username_filter {
            if !keyword.trim().is_empty() {
                query.push_str(&format!(
                    " AND (username ILIKE '%{}%' OR real_name ILIKE '%{}%')",
                    keyword.replace("'", "''"),
                    keyword.replace("'", "''")
                ));
            }
        }

        let count: (i64,) = sqlx::query_as(&query).fetch_one(pool).await.map_err(|e| {
            tracing::error!("Database error counting users: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(count.0)
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
             RETURNING id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at"
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
    ) -> Result<Option<UserEntity>, ServiceError> {
        let mut updates = vec![];

        if let Some(email) = email {
            updates.push(("email", email));
        }
        if let Some(real_name) = real_name {
            updates.push(("real_name", real_name));
        }
        if let Some(status) = status {
            updates.push(("status", status.to_string()));
        }
        if let Some(password_hash) = password_hash {
            updates.push(("password_hash", password_hash));
        }

        let set_clause = updates
            .iter()
            .enumerate()
            .map(|(i, (field, _))| format!("{} = ${}", field, i + 1))
            .collect::<Vec<_>>()
            .join(", ");

        let query = format!(
            "UPDATE users SET {} WHERE id = ${} AND deleted_at IS NULL
             RETURNING id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at",
            set_clause,
            updates.len() + 1
        );

        let mut query_builder = sqlx::query_as::<_, UserEntity>(&query);

        // Bind all parameters
        for (_, value) in updates {
            query_builder = query_builder.bind(value);
        }
        query_builder = query_builder.bind(id);

        let result = query_builder.fetch_optional(pool).await.map_err(|e| {
            tracing::error!("Database error updating user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(result)
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

    /// Update user's last login time
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), ServiceError> {
        sqlx::query(
            "UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL",
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating last login for user ID {}: {:?}", id, e);
            ServiceError::DatabaseQueryFailed
        })?;

        Ok(())
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

    /// Get user permissions (for authorization)
    pub async fn get_user_permissions(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<String>, ServiceError> {
        let sql = "SELECT permission_code FROM user_permissions WHERE user_id = $1";
        let perms: Vec<String> =
            sqlx::query_scalar(sql).bind(user_id).fetch_all(pool).await.map_err(|e| {
                tracing::error!(
                    "Database error getting permissions for user ID {}: {:?}",
                    user_id,
                    e
                );
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(perms)
    }

    pub async fn find_user_detail(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Option<UserWithRolesEntity>, ServiceError> {
        let sql = "SELECT * FROM user_with_roles WHERE id = $1";
        let user = sqlx::query_as::<_, UserWithRolesEntity>(sql)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                tracing::error!("Database error getting user detail for ID {}: {:?}", user_id, e);
                ServiceError::DatabaseQueryFailed
            })?;
        Ok(user)
    }
}
