use axum::{Router, extract::Request, middleware::Next, response::Response, routing::MethodRouter};
use sqlx::PgPool;

use crate::{
    common::error::{AppError, ServiceError},
    features::auth::{
        extractor::CurrentUser,
        permission::{PermissionService, PermissionsCheck},
    },
};

/// Router extension for permission-based routing
pub trait RouterExt<S> {
    /// Add route with permission check
    ///
    /// Examples:
    /// - Single: PermissionsCheck::Single("system:user:list")
    /// - Any: PermissionsCheck::Any(vec!["dashboard:view", "admin:all"])
    /// - All: PermissionsCheck::All(vec!["admin:delete", "admin:confirm"])
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<S>,
        permissions_check: PermissionsCheck,
    ) -> Self;
}

impl RouterExt<PgPool> for Router<PgPool> {
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<PgPool>,
        permissions_check: PermissionsCheck,
    ) -> Self {
        tracing::debug!(
            "Registering route '{}' with permission: {}",
            path,
            permissions_check.description()
        );

        self.route(
            path,
            method_router.layer(axum::middleware::from_fn(move |req: Request, next: Next| {
                let permissions_check = permissions_check.clone();
                async move { permission_middleware(req, next, permissions_check).await }
            })),
        )
    }
}

/// Permission validation middleware
///
/// Steps:
/// 1. Extract current user from request
/// 2. Get database pool
/// 3. Check user permissions (cache-first)
/// 4. Allow or deny access
async fn permission_middleware(
    request: Request,
    next: Next,
    permissions_check: PermissionsCheck,
) -> Result<Response, AppError> {
    tracing::debug!("Checking permission: {}", permissions_check.description());

    // Get current user from auth middleware
    let current_user = request.extensions().get::<CurrentUser>().cloned().ok_or_else(|| {
        tracing::error!("CurrentUser not found - auth middleware missing?");
        AppError::from(ServiceError::InvalidToken)
    })?;

    tracing::debug!(
        "Checking {} for user {} ({})",
        permissions_check.description(),
        current_user.user_id,
        current_user.username
    );

    // Check permissions with caching
    let has_permission =
        PermissionService::check_permissions(current_user.user_id, &permissions_check).await?;

    // Deny if no permission
    if !has_permission {
        tracing::warn!(
            "Permission denied: User {} ({}) lacks: {}",
            current_user.user_id,
            current_user.username,
            permissions_check.description()
        );
        return Err(AppError::from(ServiceError::PermissionDenied));
    }

    tracing::debug!(
        "Permission granted for user {} ({})",
        current_user.user_id,
        permissions_check.description()
    );

    // Continue to handler
    Ok(next.run(request).await)
}
