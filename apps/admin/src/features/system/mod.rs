pub mod menu;
pub mod role;
pub mod status;
pub mod user;

use axum::Router;
use sqlx::SqlitePool;

use crate::infra::contract::ContractRouter;
use menu::menu_routes;
use role::role_routes;
use status::status_routes;
use user::user_contract_routes;

pub fn system_contract_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .nest("/users", user_contract_routes())
        .expect("static system contract")
        .merge_router(
            Router::new()
                .nest("/menus", menu_routes())
                .nest("/roles", role_routes())
                .nest("/status", status_routes()),
        )
}
