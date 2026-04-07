use axum::{Router, extract::Request, middleware::Next, response::Response, routing::MethodRouter};

use crate::{
    auth::CurrentUser,
    error::CoreError,
    permission::{PermissionsCheck, register_permission_codes},
};

pub trait RouterExt<S> {
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<S>,
        permissions_check: PermissionsCheck,
    ) -> Self;
}

impl<S> RouterExt<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn route_with_permission(
        self,
        path: &str,
        method_router: MethodRouter<S>,
        permissions_check: PermissionsCheck,
    ) -> Self {
        register_permission_codes(permissions_check.codes());
        self.route(
            path,
            method_router.layer(axum::middleware::from_fn(move |request: Request, next: Next| {
                let permissions_check = permissions_check.clone();
                async move { permission_middleware(request, next, permissions_check).await }
            })),
        )
    }
}

async fn permission_middleware(
    request: Request,
    next: Next,
    permissions_check: PermissionsCheck,
) -> Result<Response, CoreError> {
    let current_user =
        request.extensions().get::<CurrentUser>().cloned().ok_or(CoreError::MissingAuthContext)?;

    if !permissions_check.check(&current_user) {
        return Err(CoreError::PermissionDenied);
    }

    Ok(next.run(request).await)
}
