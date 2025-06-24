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

/// A repository for handling user data in the database.
///
/// This module provides CRUD operations for users and abstracts database
/// interactions away from the service layer.
pub struct UserRepository;

impl UserRepository {
    /// Finds a single user by their ID.
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

    /// Finds a single user by their username.
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

    /// Finds a single user by their email address.
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

    /// Finds users with pagination and optional filters.
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

        // 处理状态过滤
        if let Some(status_str) = status_filter {
            match status_str {
                "1" => query.push_str(" AND status = 1"),
                "2" => query.push_str(" AND status = 2"),
                "all" => {}                             // 不添加状态过滤
                _ => query.push_str(" AND status = 1"), // 默认只显示正常用户
            }
        }

        // 处理用户名搜索
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

    /// Counts the total number of users matching the filters.
    pub async fn count_users(
        pool: &PgPool,
        username_filter: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let mut query = String::from("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL");

        // 处理状态过滤
        if let Some(status_str) = status_filter {
            match status_str {
                "1" => query.push_str(" AND status = 1"),
                "2" => query.push_str(" AND status = 2"),
                "all" => {}                             // 不添加状态过滤
                _ => query.push_str(" AND status = 1"), // 默认只显示正常用户
            }
        }

        // 处理用户名搜索
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

    /// Creates a new user with optional roles in a single transaction.
    /// This is the unified method for all user creation scenarios.
    pub async fn create_user(
        pool: &PgPool,
        request: &CreateUserRequest,
    ) -> Result<UserEntity, sqlx::Error> {
        let mut tx = pool.begin().await?;

        // 1. 创建用户
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

        // 2. 设置用户角色（如果有的话）
        if !request.role_ids.is_empty() {
            // 首先验证所有角色ID都存在
            let valid_roles = sqlx::query_as::<_, (i64,)>(
                "SELECT id FROM roles WHERE id = ANY($1) AND deleted_at IS NULL AND status = 1",
            )
            .bind(&request.role_ids)
            .fetch_all(&mut *tx)
            .await?;

            if valid_roles.len() != request.role_ids.len() {
                // 有无效的角色ID，回滚事务
                tx.rollback().await?;
                return Err(sqlx::Error::RowNotFound);
            }

            // 插入用户角色关联
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

        // 3. 提交事务
        tx.commit().await?;
        Ok(user)
    }

    /// Updates an existing user's details.
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

    /// Soft deletes a user by setting the `deleted_at` timestamp.
    /// Returns `true` if a row was affected.
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

    /// Updates the `last_login_at` timestamp for a user.
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// Retrieves all roles assigned to a specific user.
    pub async fn get_user_roles(pool: &PgPool, user_id: i64) -> Result<Vec<RoleInfo>, sqlx::Error> {
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

    /// Sets the roles for a user, replacing any existing roles.
    /// This is done within a transaction.
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

    /// Retrieves users for dropdown options
    pub async fn find_options(
        pool: &PgPool,
        status: Option<i16>, // 1, 2, 或 None(所有用户)
        q: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<(i64, String)>, sqlx::Error> {
        let mut query = String::from(
            "SELECT id, COALESCE(real_name, username) as display_name FROM users WHERE deleted_at IS NULL",
        );

        // 处理 status 过滤
        if let Some(user_status) = status {
            query.push_str(&format!(" AND status = {}", user_status));
        }

        // 处理模糊搜索
        if let Some(keyword) = q {
            query.push_str(&format!(
                " AND (username ILIKE '%{}%' OR real_name ILIKE '%{}%')",
                keyword.replace("'", "''"),
                keyword.replace("'", "''")
            ));
        }

        query.push_str(" ORDER BY display_name ASC");

        // 处理 limit
        if let Some(l) = limit {
            query.push_str(&format!(" LIMIT {}", l));
        }

        let users = sqlx::query_as(&query).fetch_all(pool).await?;

        Ok(users)
    }
}
