// Menu-related business logic (validation, combining repo methods, etc.) goes here.

use super::model::{
    CreateMenuRequest, MenuListResponse, MenuQueryParams, MenuResponse, UpdateMenuRequest,
};
use super::repo::MenuRepository;
use crate::common::api::ApiResponse;
use sqlx::PgPool;
use std::collections::HashMap;

/// 菜单服务层
pub struct MenuService;

impl MenuService {
    /// 获取菜单列表（树形结构）
    pub async fn get_menu_list(
        pool: &PgPool,
        params: MenuQueryParams,
    ) -> Result<axum::Json<ApiResponse<MenuListResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 获取所有菜单
        let menus =
            MenuRepository::find_with_conditions(pool, params.title.as_deref(), params.status)
                .await?;

        // 获取总数
        let total =
            MenuRepository::count_menus(pool, params.title.as_deref(), params.status).await?;

        // 转换为响应格式
        let menu_responses: Vec<MenuResponse> = menus.into_iter().map(MenuResponse::from).collect();

        // 构建树形结构
        let menu_tree = Self::build_menu_tree(menu_responses);

        let response = MenuListResponse { list: menu_tree, total };

        Ok(ApiResponse::success(response))
    }

    /// 根据 ID 获取菜单
    pub async fn get_menu_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<axum::Json<ApiResponse<MenuResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let menu = MenuRepository::find_by_id(pool, id).await?;

        match menu {
            Some(menu) => {
                let menu_response = MenuResponse::from(menu);
                Ok(ApiResponse::success(menu_response))
            }
            None => Ok(ApiResponse::fail(404, "菜单不存在".to_string())),
        }
    }

    /// 创建菜单
    pub async fn create_menu(
        pool: &PgPool,
        request: CreateMenuRequest,
    ) -> Result<axum::Json<ApiResponse<MenuResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 验证父菜单是否存在（如果有父菜单）
        if let Some(parent_id) = request.parent_id {
            if MenuRepository::find_by_id(pool, parent_id).await?.is_none() {
                return Ok(ApiResponse::fail(2001, "父菜单不存在".to_string()));
            }
        }

        // 创建菜单
        let menu = MenuRepository::create(
            pool,
            request.parent_id,
            &request.title,
            request.path.as_deref(),
            request.component.as_deref(),
            request.icon.as_deref(),
            request.sort_order.unwrap_or(0),
            request.status.unwrap_or(1),
        )
        .await?;

        let menu_response = MenuResponse::from(menu);
        Ok(ApiResponse::success(menu_response))
    }

    /// 更新菜单
    pub async fn update_menu(
        pool: &PgPool,
        id: i64,
        request: UpdateMenuRequest,
    ) -> Result<axum::Json<ApiResponse<MenuResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 检查菜单是否存在
        let existing_menu = MenuRepository::find_by_id(pool, id).await?;
        if existing_menu.is_none() {
            return Ok(ApiResponse::fail(404, "菜单不存在".to_string()));
        }

        // 验证父菜单是否存在（如果更新了父菜单且不是根菜单）
        if let Some(parent_id) = request.parent_id {
            if parent_id != 0 && parent_id != id {
                if MenuRepository::find_by_id(pool, parent_id).await?.is_none() {
                    return Ok(ApiResponse::fail(2001, "父菜单不存在".to_string()));
                }
            }
            // 防止将菜单设置为自己的子菜单
            if parent_id == id {
                return Ok(ApiResponse::fail(2002, "不能将菜单设置为自己的父菜单".to_string()));
            }
        }

        // 更新菜单
        let updated_menu = MenuRepository::update(
            pool,
            id,
            request.parent_id,
            request.title.as_deref(),
            request.path.as_deref(),
            request.component.as_deref(),
            request.icon.as_deref(),
            request.sort_order,
            request.status,
        )
        .await?;

        match updated_menu {
            Some(menu) => {
                let menu_response = MenuResponse::from(menu);
                Ok(ApiResponse::success(menu_response))
            }
            None => Ok(ApiResponse::fail(404, "菜单不存在".to_string())),
        }
    }

    /// 删除菜单
    pub async fn delete_menu(
        pool: &PgPool,
        id: i64,
    ) -> Result<axum::Json<ApiResponse<()>>, Box<dyn std::error::Error + Send + Sync>> {
        // 检查是否有子菜单
        let all_menus = MenuRepository::find_all(pool).await?;
        let has_children = all_menus.iter().any(|menu| menu.parent_id == Some(id));

        if has_children {
            return Ok(ApiResponse::fail(2003, "该菜单下还有子菜单，无法删除".to_string()));
        }

        let success = MenuRepository::soft_delete(pool, id).await?;

        if success {
            Ok(ApiResponse::success(()))
        } else {
            Ok(ApiResponse::fail(404, "菜单不存在".to_string()))
        }
    }

    /// 根据角色ID获取菜单（用于权限控制）
    pub async fn get_menus_by_role_ids(
        pool: &PgPool,
        role_ids: &[i64],
    ) -> Result<Vec<MenuResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let menus = MenuRepository::find_by_role_ids(pool, role_ids).await?;
        let menu_responses: Vec<MenuResponse> = menus.into_iter().map(MenuResponse::from).collect();
        Ok(menu_responses)
    }

    /// 构建菜单树
    pub fn build_menu_tree(menus: Vec<MenuResponse>) -> Vec<MenuResponse> {
        let mut menu_map: HashMap<i64, MenuResponse> = HashMap::new();

        // 创建菜单映射
        for menu in menus {
            menu_map.insert(menu.id, menu);
        }

        // 构建树结构（根菜单的parent_id为None）
        Self::find_children(&menu_map, None)
    }

    /// 递归查找子菜单
    fn find_children(
        menu_map: &HashMap<i64, MenuResponse>,
        parent_id: Option<i64>,
    ) -> Vec<MenuResponse> {
        let mut children = Vec::new();

        for (_, menu) in menu_map.iter() {
            if menu.parent_id == parent_id {
                let mut child = menu.clone();
                child.children = Self::find_children(menu_map, Some(menu.id));
                children.push(child);
            }
        }

        // 按排序字段排序
        children.sort_by(|a, b| a.sort_order.cmp(&b.sort_order));
        children
    }
}
