pub mod handler;
pub mod repo;
pub mod service;
pub mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{delete, get, put};
use handler::{delete_menu, get_menu_options, list_menus, list_module_menu_inventory, update_menu};
use rustzen_auth::capability::system_menu;
use sqlx::SqlitePool;

pub fn menu_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::ListMenus,
            AccessPolicy::Require(system_menu::LIST),
            get(list_menus),
        )
        .expect("static menu contract")
        .get(
            "/inventory",
            OperationDescriptor::ListModuleMenuInventory,
            AccessPolicy::Require(system_menu::LIST),
            get(list_module_menu_inventory),
        )
        .expect("static menu contract")
        .put(
            "/inventory/{id}",
            OperationDescriptor::UpdateMenu,
            AccessPolicy::Require(system_menu::UPDATE),
            put(update_menu),
        )
        .expect("static menu contract")
        .delete(
            "/{id}",
            OperationDescriptor::DeleteMenu,
            AccessPolicy::Require(system_menu::DELETE),
            delete(delete_menu),
        )
        .expect("static menu contract")
        .get(
            "/options",
            OperationDescriptor::GetMenuOptions,
            AccessPolicy::Require(system_menu::OPTIONS),
            get(get_menu_options),
        )
        .expect("static menu contract")
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

        let (router, _) = menu_routes().into_parts();
        let response =
            router.with_state(pool).oneshot(request).await.expect("menu router response");

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
