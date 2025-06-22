// backend/src/features/user/repo.rs

// 在这里实现所有与用户表 (sys_user) 相关的数据库操作。
// 例如:
//
// use sqlx::PgPool;
// use crate::core::errors::AppError;
// use super::model::User;
//
// pub async fn find_by_id(pool: &PgPool, id: i32) -> Result<Option<User>, AppError> {
//     // ... SQLx 查询逻辑 ...
// }

use super::model::{RoleInfo, UserEntity};
use chrono::Utc;
use sqlx::PgPool;

/// 用户数据访问层
pub struct UserRepository;

impl UserRepository {
    /// 根据 ID 获取用户
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<UserEntity>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status, 
             last_login_at, created_at, updated_at, deleted_at 
             FROM users WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// 根据用户名获取用户
    pub async fn find_by_username(
        pool: &PgPool,
        username: &str,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status, 
             last_login_at, created_at, updated_at, deleted_at 
             FROM users WHERE username = $1 AND deleted_at IS NULL",
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// 根据邮箱获取用户
    pub async fn find_by_email(
        pool: &PgPool,
        email: &str,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        let user = sqlx::query_as::<_, UserEntity>(
            "SELECT id, username, email, password_hash, real_name, avatar_url, status, 
             last_login_at, created_at, updated_at, deleted_at 
             FROM users WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// 简化的用户列表查询
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        username_filter: Option<&str>,
        status_filter: Option<i16>,
    ) -> Result<Vec<UserEntity>, sqlx::Error> {
        let users = if username_filter.is_none() && status_filter.is_none() {
            // 没有过滤条件
            sqlx::query_as::<_, UserEntity>(
                "SELECT id, username, email, password_hash, real_name, avatar_url, status, 
                 last_login_at, created_at, updated_at, deleted_at 
                 FROM users WHERE deleted_at IS NULL
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            // 有过滤条件，先简单实现
            sqlx::query_as::<_, UserEntity>(
                "SELECT id, username, email, password_hash, real_name, avatar_url, status, 
                 last_login_at, created_at, updated_at, deleted_at 
                 FROM users WHERE deleted_at IS NULL
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(users)
    }

    /// 获取用户总数
    pub async fn count_users(
        pool: &PgPool,
        _username_filter: Option<&str>,
        _status_filter: Option<i16>,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }

    /// 创建用户
    pub async fn create(
        pool: &PgPool,
        username: &str,
        email: &str,
        password_hash: &str,
        real_name: Option<&str>,
        status: i16,
    ) -> Result<UserEntity, sqlx::Error> {
        let user = sqlx::query_as::<_, UserEntity>(
            "INSERT INTO users (username, email, password_hash, real_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $6)
             RETURNING id, username, email, password_hash, real_name, avatar_url, status, 
             last_login_at, created_at, updated_at, deleted_at"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .bind(real_name)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// 更新用户
    pub async fn update(
        pool: &PgPool,
        id: i64,
        email: Option<&str>,
        real_name: Option<&str>,
        status: Option<i16>,
    ) -> Result<Option<UserEntity>, sqlx::Error> {
        // 简化实现，先查询现有用户
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
                 last_login_at, created_at, updated_at, deleted_at",
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

    /// 软删除用户
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

    /// 更新最后登录时间
    pub async fn update_last_login(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET last_login_at = $1, updated_at = $1 WHERE id = $2")
            .bind(Utc::now().naive_utc())
            .bind(id)
            .execute(pool)
            .await?;

        Ok(())
    }

    /// 获取用户的角色
    pub async fn get_user_roles(pool: &PgPool, user_id: i64) -> Result<Vec<RoleInfo>, sqlx::Error> {
        let roles = sqlx::query_as::<_, RoleInfo>(
            "SELECT r.id, r.role_name 
             FROM roles r
             INNER JOIN user_roles ur ON r.id = ur.role_id
             WHERE ur.user_id = $1 AND r.deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }

    /// 设置用户角色
    pub async fn set_user_roles(
        pool: &PgPool,
        user_id: i64,
        role_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        // 开始事务
        let mut tx = pool.begin().await?;

        // 删除现有角色
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // 添加新角色
        for role_id in role_ids {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id, created_at) VALUES ($1, $2, $3)",
            )
            .bind(user_id)
            .bind(role_id)
            .bind(Utc::now().naive_utc())
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;
        Ok(())
    }
}
