use crate::common::api::ApiResponse;
use crate::core::jwt::JWT_CONFIG;
use crate::features::auth::model::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserDetail, UserInfo,
    UserInfoResponse,
};
use crate::features::system::user::model::CreateUserRequest;
use crate::features::system::user::repo::UserRepository;
use crate::features::system::user::service::UserService;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub username: String,
    pub exp: usize, // 过期时间
    pub iat: usize, // 签发时间
}

/// 认证服务    
pub struct AuthService;

impl AuthService {
    /// 用户注册
    pub async fn register(
        pool: &PgPool,
        request: RegisterRequest,
    ) -> Result<axum::Json<ApiResponse<RegisterResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        // 验证用户名是否已存在
        if let Some(_) = UserRepository::find_by_username(pool, &request.username).await? {
            return Ok(ApiResponse::fail(2001, "用户名已存在".to_string()));
        }

        // 验证邮箱是否已存在
        if let Some(_) = UserRepository::find_by_email(pool, &request.email).await? {
            return Ok(ApiResponse::fail(2002, "邮箱已存在".to_string()));
        }

        // 创建用户请求
        let create_request = CreateUserRequest {
            username: request.username.clone(),
            email: request.email.clone(),
            password: request.password,
            real_name: request.real_name.clone(),
            status: Some(1),  // 默认启用状态
            role_ids: vec![], // 注册时不分配角色，可以后续管理员分配
        };

        // 调用用户服务创建用户
        match UserService::create_user(pool, create_request).await? {
            response if response.code == 200 => {
                if let Some(user_data) = &response.data {
                    let register_response = RegisterResponse {
                        user_id: user_data.id,
                        username: user_data.username.clone(),
                        email: user_data.email.clone(),
                        real_name: user_data.real_name.clone(),
                        message: "注册成功".to_string(),
                    };
                    Ok(ApiResponse::success(register_response))
                } else {
                    Ok(ApiResponse::fail(500, "注册失败：无法获取用户信息".to_string()))
                }
            }
            error_response => {
                Ok(ApiResponse::fail(error_response.code, error_response.message.clone()))
            }
        }
    }

    /// 用户登录
    pub async fn login(
        pool: &PgPool,
        request: LoginRequest,
    ) -> Result<axum::Json<ApiResponse<LoginResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let user = UserService::verify_login(pool, &request.username, &request.password).await?;

        match user {
            Some(user) => {
                // 生成 JWT token
                let now = Utc::now();
                let exp = (now + Duration::seconds(JWT_CONFIG.expiration)).timestamp() as usize;
                let iat = now.timestamp() as usize;

                let claims =
                    Claims { sub: user.id.to_string(), username: user.username.clone(), exp, iat };

                let token = encode(
                    &Header::default(),
                    &claims,
                    &EncodingKey::from_secret(JWT_CONFIG.secret.as_bytes()),
                )?;

                // 获取用户角色
                let roles = UserRepository::get_user_roles(pool, user.id).await?;
                let role_names: Vec<String> = roles.iter().map(|r| r.role_name.clone()).collect();

                let response = LoginResponse {
                    access_token: token,
                    expires_in: JWT_CONFIG.expiration,
                    user_info: UserInfo {
                        id: user.id,
                        username: user.username,
                        real_name: user.real_name,
                        roles: role_names,
                    },
                };

                Ok(ApiResponse::success(response))
            }
            None => Ok(ApiResponse::fail(1001, "用户名或密码错误".to_string())),
        }
    }

    /// 获取用户信息
    pub async fn get_user_info(
        pool: &PgPool,
        user_id: i64,
    ) -> Result<axum::Json<ApiResponse<UserInfoResponse>>, Box<dyn std::error::Error + Send + Sync>>
    {
        let user = UserRepository::find_by_id(pool, user_id).await?;

        match user {
            Some(user) => {
                // 获取用户菜单
                let menus = UserService::get_user_menus(pool, user_id).await?;
                let menu_tree = Self::build_menu_tree(menus);

                let response = UserInfoResponse {
                    user: UserDetail {
                        id: user.id,
                        username: user.username,
                        real_name: user.real_name,
                        email: user.email,
                        avatar_url: user.avatar_url,
                        status: user.status,
                        last_login_at: user
                            .last_login_at
                            .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc)),
                    },
                    menus: menu_tree,
                };

                Ok(ApiResponse::success(response))
            }
            None => Ok(ApiResponse::fail(1002, "用户不存在".to_string())),
        }
    }

    /// 验证 JWT token
    pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_CONFIG.secret.as_bytes()),
            &validation,
        )?;

        Ok(token_data.claims)
    }

    /// 构建菜单树
    fn build_menu_tree(
        menus: Vec<crate::features::system::menu::model::MenuResponse>,
    ) -> Vec<crate::features::system::menu::model::MenuResponse> {
        use std::collections::HashMap;

        // 创建菜单映射
        let mut menu_map: HashMap<i64, crate::features::system::menu::model::MenuResponse> =
            HashMap::new();
        for menu in menus {
            menu_map.insert(menu.id, menu);
        }

        // 构建树结构（根菜单的parent_id为None）
        Self::find_children(&menu_map, None)
    }

    /// 递归查找子菜单
    fn find_children(
        menu_map: &std::collections::HashMap<
            i64,
            crate::features::system::menu::model::MenuResponse,
        >,
        parent_id: Option<i64>,
    ) -> Vec<crate::features::system::menu::model::MenuResponse> {
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
