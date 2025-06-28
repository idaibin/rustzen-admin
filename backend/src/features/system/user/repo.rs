// backend/src/features/user/repo.rs

// This module implements all database operations related to the `users` table.
//
// Functions in this module are designed to be simple, direct database
// interactions. All logic, such as mapping to response models, checking
// permissions, or combining multiple operations, should be handled in the
// service layer.

use super::model::{CreateUserRequest, RoleInfo, UserEntity};
use chrono::Utc;
use sqlx::PgPool;

/// User repository for database operations
pub struct UserRepository;

impl UserRepository {
    /// Find user by ID
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<UserEntity>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at
             FROM users WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by username
    pub async fn find_by_username(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at
             FROM users WHERE username = $1 AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at
             FROM users WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Find users with pagination and filters
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        username_filter: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<Vec<UserEntity>, sqlx::Error> {
        let mut query = String::from(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at
             FROM users WHERE deleted_at IS NULL",
        );

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

        query.push_str(" ORDER BY created_at DESC LIMIT $1 OFFSET $2");

        let users = sqlx::query_as::<_, UserEntity>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok(users)
    }

    /// Count users matching filters
    pub async fn count_users(
        pool: &PgPool,
        username_filter: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
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

        let count: (i64,) = sqlx::query_as(&query).fetch_one(pool).await?;
        Ok(count.0)
    }

    /// Create new user with optional roles (unified method)
    pub async fn create_user(
        pool: &PgPool,
        request: &CreateUserRequest,
    ) -> Result<UserEntity, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // Create user
        let user = sqlx::query_as::<_, UserEntity>(
            "INSERT INTO users (username, email, password_hash, real_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $6)
             RETURNING id, username, email, password_hash, real_name, avatar_url, status,
             last_login_at, created_at, updated_at"
        )
        .bind(&request.username)
        .bind(&request.email)
        .bind(&request.password)
        .bind(request.real_name.as_deref())
        .bind(request.status.unwrap_or(1))
        .bind(Utc::now().naive_utc())
        .fetch_one(&mut *tx)
        .await?;

        // Set user roles if provided
        if !request.role_ids.is_empty() {
            // Validate all role IDs exist
            let valid_roles = sqlx::query_as::<_, (i64,)>(
                "SELECT id FROM roles WHERE id = ANY($1) AND deleted_at IS NULL AND status = 1",
            )
            .bind(&request.role_ids)
            .fetch_all(&mut *tx)
            .await?;

            if valid_roles.len() != request.role_ids.len() {
                // Invalid role IDs, rollback transaction
                tx.rollback().await?;
                return Err(sqlx::Error::RowNotFound);
            }

            // Insert user role associations
            let mut query_builder =
                "INSERT INTO user_roles (user_id, role_id, created_at) VALUES ".to_string();
            let now = Utc::now().naive_utc();
            for (i, role_id) in request.role_ids.iter().enumerate() {
                if i > 0 {
                    query_builder.push_str(", ");
                }
                query_builder.push_str(&format!("({}, {}, '{}')", user.id, role_id, now));
            }

            sqlx::query(&query_builder).execute(&mut *tx).await?;
        }

        // Commit transaction
        tx.commit().await?;
        Ok(user)
    }

    /// Update existing user
    pub async fn update(
        pool: &PgPool,
        id: i64,
        email: Option<&str>,
        real_name: Option<&str>,
        status: Option<i16>,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let existing = Self::find_by_id(pool, id).await?;
        if let Some(existing_user) = existing {
            let updated_email = email.unwrap_or(&existing_user.email);
            let updated_real_name = real_name.or(existing_user.real_name.as_deref());
            let updated_status = status.unwrap_or(existing_user.status);

            let user = sqlx::query_as::<_, UserEntity>(
                "UPDATE users
                 SET email = $2, real_name = $3, status = $4, updated_at = $5
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id, username, email, password_hash, real_name, avatar_url, status,
                 last_login_at, created_at, updated_at",
            )
            .bind(id)
            .bind(updated_email)
            .bind(updated_real_name)
            .bind(updated_status)
            .bind(Utc::now().naive_utc())
            .fetch_optional(pool)
            .await?;

            Ok(user)
        } else {
            Ok(None)
        }
    }

    /// Soft delete user
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE users SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Update last login timestamp
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Get user roles with info
    pub async fn get_user_role_infos(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<RoleInfo>, sqlx::Error> {
        let roles = sqlx::query_as::<_, RoleInfo>(
            "SELECT r.id, r.role_name
             FROM roles r
             INNER JOIN user_roles ur ON r.id = ur.role_id
             WHERE ur.user_id = $1 AND r.deleted_at IS NULL AND r.status = 1",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }

    pub async fn get_user_role_ids(pool: &PgPool, user_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let roles =
            sqlx::query_as::<_, (i64,)>("SELECT role_id FROM user_roles WHERE user_id = $1")
                .bind(user_id)
                .fetch_all(pool)
                .await?;

        Ok(roles.iter().map(|r| r.0).collect())
    }

    /// Set user roles (replaces existing)
    pub async fn set_user_roles(
        pool: &PgPool,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        if !role_ids.is_empty() {
            let mut query_builder =
                "INSERT INTO user_roles (user_id, role_id, created_at) VALUES ".to_string();
            let now = Utc::now().naive_utc();
            for (i, role_id) in role_ids.iter().enumerate() {
                if i > 0 {
                    query_builder.push_str(", ");
                }
                query_builder.push_str(&format!("({}, {}, '{}')", user_id, role_id, now));
            }

            sqlx::query(&query_builder).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Get users for dropdown options
    pub async fn find_options(
        pool: &PgPool,
        status: Option<i16>, // 1, 2, or None (all users)
        q: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let mut query = String::from(
            "SELECT id, COALESCE(real_name, username) as display_name FROM users WHERE deleted_at IS NULL",
        );

        // Handle status filter
        if let Some(user_status) = status {
            query.push_str(&format!(" AND status = {}", user_status));
        }

        // Handle search
        if let Some(keyword) = q {
            query.push_str(&format!(
                " AND (username ILIKE '%{}%' OR real_name ILIKE '%{}%')",
                keyword.replace("'", "''"),
                keyword.replace("'", "''")
            ));
        }

        query.push_str(" ORDER BY display_name ASC");

        // Handle limit
        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let users = sqlx::query_as(&query).fetch_all(pool).await?;

        Ok(users)
    }

    /// Get user permissions (placeholder - to be implemented)
    pub async fn get_user_permissions(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        // Placeholder: return empty until role permissions are implemented
        // TODO: Implement after adding permissions field to roles table
        let _ = (pool, user_id); // Avoid compiler warning
        Ok(vec![])
    }
}
