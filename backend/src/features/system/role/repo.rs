// Database operations related to the `sys_role` table go here.

use super::model::RoleEntity;
use chrono::Utc;
use sqlx::PgPool;

/// 角色数据访问层
pub struct RoleRepository;

impl RoleRepository {
    /// 根据 ID 获取角色
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<RoleEntity>, sqlx::Error> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "SELECT id, role_name, status, 
             created_at, updated_at, deleted_at 
             FROM roles WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    /// 根据角色代码获取角色
    pub async fn find_by_role_name(
        pool: &PgPool,
        role_name: &str,
    ) -> Result<Option<RoleEntity>, sqlx::Error> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "SELECT id, role_name, status, 
             created_at, updated_at, deleted_at 
             FROM roles WHERE role_name = $1 AND deleted_at IS NULL",
        )
        .bind(role_name)
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    /// 分页查询角色
    pub async fn find_with_pagination(
        pool: &PgPool,
        offset: i64,
        limit: i64,
        role_name_filter: Option<&str>,
        status_filter: Option<i16>,
    ) -> Result<Vec<RoleEntity>, sqlx::Error> {
        let roles = if role_name_filter.is_none() && status_filter.is_none() {
            // 没有过滤条件
            sqlx::query_as::<_, RoleEntity>(
                "SELECT id, role_name, status, 
                 created_at, updated_at, deleted_at 
                 FROM roles WHERE deleted_at IS NULL
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            // 有过滤条件，先简单实现
            sqlx::query_as::<_, RoleEntity>(
                "SELECT id, role_name, status, 
                 created_at, updated_at, deleted_at 
                 FROM roles WHERE deleted_at IS NULL
                 ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };

        Ok(roles)
    }

    /// 获取角色总数
    pub async fn count_roles(
        pool: &PgPool,
        _role_name_filter: Option<&str>,
        _status_filter: Option<i16>,
    ) -> Result<i64, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM roles WHERE deleted_at IS NULL")
            .fetch_one(pool)
            .await?;

        Ok(count.0)
    }

    /// 创建角色
    pub async fn create(
        pool: &PgPool,
        role_name: &str,
        status: i16,
    ) -> Result<RoleEntity, sqlx::Error> {
        let role = sqlx::query_as::<_, RoleEntity>(
            "INSERT INTO roles (role_name, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4)
             RETURNING id, role_name, status, 
             created_at, updated_at, deleted_at",
        )
        .bind(role_name)
        .bind(status)
        .bind(Utc::now().naive_utc())
        .fetch_one(pool)
        .await?;

        Ok(role)
    }

    /// 更新角色
    pub async fn update(
        pool: &PgPool,
        id: i64,
        role_name: Option<&str>,
        status: Option<i16>,
    ) -> Result<Option<RoleEntity>, sqlx::Error> {
        // 简化实现，先查询现有角色
        let existing = Self::find_by_id(pool, id).await?;
        if let Some(existing_role) = existing {
            let updated_role_name = role_name.unwrap_or(&existing_role.role_name);
            let updated_status = status.unwrap_or(existing_role.status);

            let role = sqlx::query_as::<_, RoleEntity>(
                "UPDATE roles 
                 SET role_name = $2, status = $3, updated_at = $4
                 WHERE id = $1 AND deleted_at IS NULL
                 RETURNING id, role_name, status, 
                 created_at, updated_at, deleted_at",
            )
            .bind(id)
            .bind(updated_role_name)
            .bind(updated_status)
            .bind(Utc::now().naive_utc())
            .fetch_optional(pool)
            .await?;

            Ok(role)
        } else {
            Ok(None)
        }
    }

    /// 软删除角色
    pub async fn soft_delete(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE roles SET deleted_at = $1, updated_at = $1 WHERE id = $2 AND deleted_at IS NULL"
        )
        .bind(Utc::now().naive_utc())
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取角色的菜单ID列表
    pub async fn get_role_menu_ids(pool: &PgPool, role_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let menu_ids: Vec<(i64,)> =
            sqlx::query_as("SELECT menu_id FROM role_menus WHERE role_id = $1")
                .bind(role_id)
                .fetch_all(pool)
                .await?;

        Ok(menu_ids.into_iter().map(|(id,)| id).collect())
    }

    /// 设置角色菜单
    pub async fn set_role_menus(
        pool: &PgPool,
        role_id: i64,
        menu_ids: &[i64],
    ) -> Result<(), sqlx::Error> {
        // 开始事务
        let mut tx = pool.begin().await?;

        // 删除现有菜单关联
        sqlx::query("DELETE FROM role_menus WHERE role_id = $1")
            .bind(role_id)
            .execute(&mut *tx)
            .await?;

        // 添加新菜单关联
        for menu_id in menu_ids {
            sqlx::query(
                "INSERT INTO role_menus (role_id, menu_id, created_at) VALUES ($1, $2, $3)",
            )
            .bind(role_id)
            .bind(menu_id)
            .bind(Utc::now().naive_utc())
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;
        Ok(())
    }

    /// 获取所有角色（用于下拉选择等）
    pub async fn find_all_active(pool: &PgPool) -> Result<Vec<RoleEntity>, sqlx::Error> {
        let roles = sqlx::query_as::<_, RoleEntity>(
            "SELECT id, role_name, status, 
             created_at, updated_at, deleted_at 
             FROM roles WHERE deleted_at IS NULL AND status = 1
             ORDER BY role_name ASC",
        )
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }
}
