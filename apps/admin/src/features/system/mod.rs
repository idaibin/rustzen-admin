pub mod menu;
pub mod role;
pub mod status;
pub mod user;

use sqlx::SqlitePool;

use crate::infra::contract::ContractRouter;
use role::role_routes;
use user::user_contract_routes;

pub fn system_contract_routes() -> ContractRouter<SqlitePool> {
    let router = ContractRouter::new()
        .nest("/users", user_contract_routes())
        .expect("static system contract")
        .nest("/roles", role_routes())
        .expect("static system contract");
    router
        .nest("/menus", menu::menu_routes())
        .expect("static system contract")
        .nest("/status", status::status_routes())
        .expect("static system contract")
}
