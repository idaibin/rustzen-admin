// backend/src/features/user/service.rs

// 在这里实现用户相关的业务逻辑。
// 例如:
// - 校验用户创建请求的参数。
// - 组合 repo 层的方法来完成一个完整的业务操作。
// - 处理权限、发送通知等。

use super::model::{
    CreateUserRequest, UpdateUserRequest, UserEntity, UserListResponse, UserQueryParams,
    UserResponse,
};
use super::repo::UserRepository;
use crate::common::api::ApiResponse;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

/// 用户服务层
pub struct UserService;

impl UserService {
    /// 获取用户列表
    pub async fn get_user_list(
        pool: &PgPool,
        params: UserQueryParams,
    ) -> Result<axum::Json<ApiResponse<UserListResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let page = params.current.unwrap_or(1).max(1);
        let page_size = params.page_size.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) * page_size;
        // 获取用户列表
        let users = UserRepository::find_with_pagination(
            pool,
            offset,
            page_size,
            params.username.as_deref(),
            params.status,
        )
        .await?;

        // 获取总数
        let total =
            UserRepository::count_users(pool, params.username.as_deref(), params.status).await?;

        // 转换为响应格式并填充角色信息
        let mut user_responses = Vec::new();
        for user in users {
            let roles = UserRepository::get_user_roles(pool, user.id).await?;
            let mut user_response = UserResponse::from(user);
            user_response.roles = roles;
            user_responses.push(user_response);
        }

        let response = UserListResponse { list: user_responses, total, page, page_size };

        Ok(ApiResponse::success(response))
    }

    /// 根据 ID 获取用户
    pub async fn get_user_by_id(
        pool: &PgPool,
        id: i64,
    ) -> Result<axum::Json<ApiResponse<UserResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let user = UserRepository::find_by_id(pool, id).await?;

        match user {
            Some(user) => {
                let roles = UserRepository::get_user_roles(pool, user.id).await?;
                let mut user_response = UserResponse::from(user);
                user_response.roles = roles;
                Ok(ApiResponse::success(user_response))
            }
            None => Ok(ApiResponse::fail(404, "用户不存在".to_string())),
        }
    }

    /// 创建用户
    pub async fn create_user(
        pool: &PgPool,
        request: CreateUserRequest,
    ) -> Result<axum::Json<ApiResponse<UserResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 验证用户名是否已存在
        if let Some(_) = UserRepository::find_by_username(pool, &request.username).await? {
            return Ok(ApiResponse::fail(2001, "用户名已存在".to_string()));
        }

        // 验证邮箱是否已存在
        if let Some(_) = UserRepository::find_by_email(pool, &request.email).await? {
            return Ok(ApiResponse::fail(2002, "邮箱已存在".to_string()));
        }

        // 哈希密码
        let password_hash = Self::hash_password(&request.password);

        // 创建用户
        let user = UserRepository::create(
            pool,
            &request.username,
            &request.email,
            &password_hash,
            request.real_name.as_deref(),
            request.status.unwrap_or(1),
        )
        .await?;

        // 设置用户角色
        if !request.role_ids.is_empty() {
            UserRepository::set_user_roles(pool, user.id, &request.role_ids).await?;
        }

        // 获取角色信息
        let roles = UserRepository::get_user_roles(pool, user.id).await?;
        let mut user_response = UserResponse::from(user);
        user_response.roles = roles;

        Ok(ApiResponse::success(user_response))
    }

    /// 更新用户
    pub async fn update_user(
        pool: &PgPool,
        id: i64,
        request: UpdateUserRequest,
    ) -> Result<axum::Json<ApiResponse<UserResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 检查用户是否存在
        let existing_user = UserRepository::find_by_id(pool, id).await?;
        if existing_user.is_none() {
            return Ok(ApiResponse::fail(404, "用户不存在".to_string()));
        }

        // 如果更新邮箱，检查邮箱是否已被其他用户使用
        if let Some(ref email) = request.email {
            if let Some(existing_email_user) = UserRepository::find_by_email(pool, email).await? {
                if existing_email_user.id != id {
                    return Ok(ApiResponse::fail(2002, "邮箱已存在".to_string()));
                }
            }
        }

        // 更新用户基本信息
        let updated_user = UserRepository::update(
            pool,
            id,
            request.email.as_deref(),
            request.real_name.as_deref(),
            request.status,
        )
        .await?;

        match updated_user {
            Some(user) => {
                // 更新角色
                if let Some(role_ids) = request.role_ids {
                    UserRepository::set_user_roles(pool, user.id, &role_ids).await?;
                }

                // 获取角色信息
                let roles = UserRepository::get_user_roles(pool, user.id).await?;
                let mut user_response = UserResponse::from(user);
                user_response.roles = roles;

                Ok(ApiResponse::success(user_response))
            }
            None => Ok(ApiResponse::fail(404, "用户不存在".to_string())),
        }
    }

    /// 删除用户
    pub async fn delete_user(
        pool: &PgPool,
        id: i64,
    ) -> Result<axum::Json<ApiResponse<()>>, Box<dyn std::error::Error + Send + Sync>> {
        let success = UserRepository::soft_delete(pool, id).await?;

        if success {
            Ok(ApiResponse::success(()))
        } else {
            Ok(ApiResponse::fail(404, "用户不存在".to_string()))
        }
    }

    /// 验证用户登录
    pub async fn verify_login(
        pool: &PgPool,
        username: &str,
        password: &str,
    ) -> Result<Option<UserEntity>, Box<dyn std::error::Error + Send + Sync>> {
        let user = UserRepository::find_by_username(pool, username).await?;

        match user {
            Some(user) => {
                if Self::verify_password(password, &user.password_hash) {
                    // 更新最后登录时间
                    UserRepository::update_last_login(pool, user.id).await?;
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// 获取用户的菜单权限
    pub async fn get_user_menus(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<
        Vec<crate::features::system::menu::model::MenuResponse>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        // 获取用户角色
        let roles = UserRepository::get_user_roles(pool, user_id).await?;
        let role_ids: Vec<i64> = roles.iter().map(|r| r.id).collect();

        if role_ids.is_empty() {
            return Ok(vec![]);
        }

        // 获取角色对应的菜单
        let menus = crate::features::system::menu::service::MenuService::get_menus_by_role_ids(
            pool, &role_ids,
        )
        .await?;
        Ok(menus)
    }

    /// 哈希密码
    fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 验证密码
    fn verify_password(password: &str, hash: &str) -> bool {
        let password_hash = Self::hash_password(password);
        password_hash == hash
    }
}
