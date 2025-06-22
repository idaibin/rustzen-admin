// Role-related business logic (validation, combining repo methods, etc.) goes here.

use super::model::{
    CreateRoleRequest, RoleListResponse, RoleQueryParams, RoleResponse, UpdateRoleRequest,
};
use super::repo::RoleRepository;
use crate::common::api::ApiResponse;
use sqlx::PgPool;

/// 角色服务层
pub struct RoleService;

impl RoleService {
    /// 获取角色列表
    pub async fn get_role_list(
        pool: &PgPool,
        params: RoleQueryParams,
    ) -> Result<axum::Json<ApiResponse<RoleListResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let page = params.current.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) * page_size;

        // 获取角色列表
        let roles = RoleRepository::find_with_pagination(
            pool,
            offset,
            page_size,
            params.role_name.as_deref(),
            params.status,
        )
        .await?;

        // 获取总数
        let total =
            RoleRepository::count_roles(pool, params.role_name.as_deref(), params.status).await?;

        // 转换为响应格式并填充菜单信息
        let mut role_responses = Vec::new();
        for role in roles {
            let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
            let mut role_response = RoleResponse::from(role);
            role_response.menu_ids = menu_ids;
            role_responses.push(role_response);
        }

        let response = RoleListResponse { list: role_responses, total, page, page_size };

        Ok(ApiResponse::success(response))
    }

    /// 根据 ID 获取角色
    pub async fn get_role_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<axum::Json<ApiResponse<RoleResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let role = RoleRepository::find_by_id(pool, id).await?;

        match role {
            Some(role) => {
                let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
                let mut role_response = RoleResponse::from(role);
                role_response.menu_ids = menu_ids;
                Ok(ApiResponse::success(role_response))
            }
            None => Ok(ApiResponse::fail(404, "角色不存在".to_string())),
        }
    }

    /// 创建角色
    pub async fn create_role(
        pool: &PgPool,
        request: CreateRoleRequest,
    ) -> Result<axum::Json<ApiResponse<RoleResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 验证角色代码是否已存在
        if let Some(_) = RoleRepository::find_by_role_name(pool, &request.role_name).await? {
            return Ok(ApiResponse::fail(2001, "角色代码已存在".to_string()));
        }

        // 创建角色
        let role =
            RoleRepository::create(pool, &request.role_name, request.status.unwrap_or(1)).await?;

        // 设置角色菜单
        if !request.menu_ids.is_empty() {
            RoleRepository::set_role_menus(pool, role.id, &request.menu_ids).await?;
        }

        // 获取菜单信息
        let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
        let mut role_response = RoleResponse::from(role);
        role_response.menu_ids = menu_ids;

        Ok(ApiResponse::success(role_response))
    }

    /// 更新角色
    pub async fn update_role(
        pool: &PgPool,
        id: i64,
        request: UpdateRoleRequest,
    ) -> Result<axum::Json<ApiResponse<RoleResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 检查角色是否存在
        let existing_role = RoleRepository::find_by_id(pool, id).await?;
        if existing_role.is_none() {
            return Ok(ApiResponse::fail(404, "角色不存在".to_string()));
        }

        // 如果更新角色代码，检查角色代码是否已被其他角色使用
        if let Some(ref role_name) = request.role_name {
            if let Some(existing_role) = RoleRepository::find_by_role_name(pool, role_name).await? {
                if existing_role.id != id {
                    return Ok(ApiResponse::fail(2001, "角色代码已存在".to_string()));
                }
            }
        }

        // 更新角色基本信息
        let updated_role =
            RoleRepository::update(pool, id, request.role_name.as_deref(), request.status).await?;

        match updated_role {
            Some(role) => {
                // 更新菜单
                if let Some(menu_ids) = request.menu_ids {
                    RoleRepository::set_role_menus(pool, role.id, &menu_ids).await?;
                }

                // 获取菜单信息
                let menu_ids = RoleRepository::get_role_menu_ids(pool, role.id).await?;
                let mut role_response = RoleResponse::from(role);
                role_response.menu_ids = menu_ids;

                Ok(ApiResponse::success(role_response))
            }
            None => Ok(ApiResponse::fail(404, "角色不存在".to_string())),
        }
    }

    /// 删除角色
    pub async fn delete_role(
        pool: &PgPool,
        id: i64,
    ) -> Result<axum::Json<ApiResponse<()>>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否有用户关联此角色
        let user_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM user_roles WHERE role_id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;

        if user_count.0 > 0 {
            return Ok(ApiResponse::fail(2002, "该角色下还有用户，无法删除".to_string()));
        }

        let success = RoleRepository::soft_delete(pool, id).await?;

        if success {
            Ok(ApiResponse::success(()))
        } else {
            Ok(ApiResponse::fail(404, "角色不存在".to_string()))
        }
    }

    /// 获取所有可用角色（用于下拉选择等）
    pub async fn get_all_active_roles(
        pool: &PgPool,
    ) -> Result<Vec<super::model::Role>, Box<dyn std::error::Error + Send + Sync>> {
        let roles = RoleRepository::find_all_active(pool).await?;
        let role_list: Vec<super::model::Role> = roles.into_iter().map(|r| r.into()).collect();
        Ok(role_list)
    }

    /// 设置角色菜单权限
    pub async fn set_role_menus(
        pool: &PgPool,
        role_id: i64,
        menu_ids: Vec<i64>,
    ) -> Result<axum::Json<ApiResponse<()>>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查角色是否存在
        if RoleRepository::find_by_id(pool, role_id).await?.is_none() {
            return Ok(ApiResponse::fail(404, "角色不存在".to_string()));
        }

        // 设置角色菜单
        RoleRepository::set_role_menus(pool, role_id, &menu_ids).await?;

        Ok(ApiResponse::success(()))
    }

    /// 获取角色菜单权限
    pub async fn get_role_menus(
        pool: &PgPool,
        role_id: i64,
    ) -> Result<axum::Json<ApiResponse<Vec<i64>>>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查角色是否存在
        if RoleRepository::find_by_id(pool, role_id).await?.is_none() {
            return Ok(ApiResponse::fail(404, "角色不存在".to_string()));
        }

        // 获取角色菜单
        let menu_ids = RoleRepository::get_role_menu_ids(pool, role_id).await?;

        Ok(ApiResponse::success(menu_ids))
    }
}
