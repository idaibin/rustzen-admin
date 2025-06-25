// src/features/user/routes.rs

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post, put},
};
use sqlx::PgPool;

use super::model::{
    CreateUserRequest, UpdateUserRequest, UserListResponse, UserQueryParams, UserResponse,
};
use super::service::UserService;
use crate::{
    common::{
        api::{ApiResponse, AppResult, OptionItem, OptionsQuery},
        router_ext::RouterExt,
    },
    core::password::PasswordUtils,
    features::auth::permission::PermissionsCheck,
};

/// User management routes with permission examples:
/// - Single: PermissionsCheck::Single("system:user:list")
/// - Any: PermissionsCheck::Any(vec!["system:user:create", "admin:full"])
/// - All: PermissionsCheck::All(vec!["system:user:get", "system:user:update"])
pub fn user_routes() -> Router<PgPool> {
    Router::new()
        // List users - single permission
        .route_with_permission(
            "/",
            get(get_user_list),
            PermissionsCheck::Single("system:user:list"),
        )
        // Create user - any permission (OR logic)
        .route_with_permission(
            "/",
            post(create_user),
            PermissionsCheck::Any(vec!["system:user:create", "admin:full"]),
        )
        // User options - single permission
        .route_with_permission(
            "/options",
            get(get_user_options),
            PermissionsCheck::Single("system:user:options"),
        )
        // Status options - no permission required
        .route("/status-options", get(get_user_status_options))
        // Get user - single permission
        .route_with_permission(
            "/{id}",
            get(get_user_by_id),
            PermissionsCheck::Single("system:user:get"),
        )
        // Update user - all permissions (AND logic)
        .route_with_permission(
            "/{id}",
            put(update_user),
            PermissionsCheck::All(vec!["system:user:get", "system:user:update"]),
        )
        // Delete user - single permission
        .route_with_permission(
            "/{id}",
            delete(delete_user),
            PermissionsCheck::Single("system:user:delete"),
        )
}

/// Get paginated user list with filtering
/// Query params: current, page_size, username, status
async fn get_user_list(
    State(pool): State<PgPool>,
    Query(params): Query<UserQueryParams>,
) -> AppResult<Json<ApiResponse<UserListResponse>>> {
    tracing::info!(
        "Get user list: page={}, size={}, filter={:?}, status={:?}",
        params.current.unwrap_or(1),
        params.page_size.unwrap_or(10),
        params.username,
        params.status
    );

    let user_list = UserService::get_user_list(&pool, params).await?;

    tracing::info!(
        "User list retrieved: total={}, returned={}",
        user_list.total,
        user_list.list.len()
    );

    Ok(ApiResponse::success(user_list))
}

/// Get user by ID with roles
async fn get_user_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    tracing::info!("Get user by ID: {}", id);

    let user = UserService::get_user_by_id(&pool, id).await?;

    tracing::info!(
        "User retrieved: id={}, username={}, roles={}",
        user.id,
        user.username,
        user.roles.len()
    );

    Ok(ApiResponse::success(user))
}

/// Create new user (admin endpoint)
/// Body: username, email, password, real_name, status, role_ids
async fn create_user(
    State(pool): State<PgPool>,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    tracing::info!(
        "Create user: username={}, email={}, roles={:?}",
        request.username,
        request.email,
        request.role_ids
    );

    // Hash password for security
    let create_request = CreateUserRequest {
        username: request.username,
        email: request.email,
        password: PasswordUtils::hash_password(&request.password)?,
        real_name: request.real_name,
        status: request.status,
        role_ids: request.role_ids,
    };

    let new_user = UserService::create_user(&pool, &create_request).await?;

    tracing::info!(
        "User created: id={}, username={}, roles={}",
        new_user.id,
        new_user.username,
        new_user.roles.len()
    );

    Ok(ApiResponse::success(new_user))
}

/// Update user information
/// Body: email, real_name, status, role_ids (all optional)
async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Json<ApiResponse<UserResponse>>> {
    tracing::info!(
        "Update user {}: email={:?}, name={:?}, status={:?}, roles={:?}",
        id,
        request.email,
        request.real_name,
        request.status,
        request.role_ids
    );

    let updated_user = UserService::update_user(&pool, id, request).await?;

    tracing::info!(
        "User updated: id={}, username={}, roles={}",
        updated_user.id,
        updated_user.username,
        updated_user.roles.len()
    );

    Ok(ApiResponse::success(updated_user))
}

/// Soft delete user
async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<Json<ApiResponse<()>>> {
    tracing::info!("Delete user: {}", id);

    UserService::delete_user(&pool, id).await?;

    tracing::info!("User deleted: {}", id);

    Ok(ApiResponse::success(()))
}

/// Get user options for dropdowns
/// Query params: q (search), limit, status
async fn get_user_options(
    State(pool): State<PgPool>,
    query: Query<OptionsQuery>,
) -> AppResult<Json<ApiResponse<Vec<OptionItem<i64>>>>> {
    tracing::debug!("User options: q={:?}, limit={:?}", query.q, query.limit);

    let options = UserService::get_user_options(&pool, query).await?;

    tracing::debug!("User options returned: {}", options.len());

    Ok(ApiResponse::success(options))
}

/// Get predefined user status options
async fn get_user_status_options() -> AppResult<Json<ApiResponse<Vec<OptionItem<i16>>>>> {
    let options = vec![
        OptionItem { label: "Normal".to_string(), value: 1 },
        OptionItem { label: "Disabled".to_string(), value: 2 },
    ];

    Ok(ApiResponse::success(options))
}
