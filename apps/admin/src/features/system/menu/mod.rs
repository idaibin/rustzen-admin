pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use axum::{
    Router,
    routing::{delete, get, put},
};
use handler::{delete_menu, get_menu_options, list_menus, list_module_menu_inventory, update_menu};
use rustzen_auth::{
    capability::system_menu,
    permission::{PermissionsCheck, RouterExt},
};
use sqlx::SqlitePool;

pub fn menu_routes() -> Router<SqlitePool> {
    Router::new()
        .route_with_permission("/", get(list_menus), PermissionsCheck::Require(system_menu::LIST))
        .route_with_permission(
            "/inventory",
            get(list_module_menu_inventory),
            PermissionsCheck::Require(system_menu::LIST),
        )
        .route_with_permission(
            "/{id}",
            put(update_menu),
            PermissionsCheck::Require(system_menu::UPDATE),
        )
        .route_with_permission(
            "/{id}",
            delete(delete_menu),
            PermissionsCheck::Require(system_menu::DELETE),
        )
        .route_with_permission(
            "/options",
            get(get_menu_options),
            PermissionsCheck::Require(system_menu::OPTIONS),
        )
}

#[cfg(test)]
mod tests {
    use super::menu_routes;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use rustzen_auth::auth::CurrentUser;
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    #[tokio::test]
    async fn permission_definitions_cannot_be_created_manually() {
        let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("lazy sqlite pool");
        let mut request = Request::builder()
            .method(Method::POST)
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .expect("menu create request");
        request.extensions_mut().insert(CurrentUser::new(1, "owner", ["*".to_string()], true));

        let response =
            menu_routes().with_state(pool).oneshot(request).await.expect("menu router response");

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
