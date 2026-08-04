pub mod menu;
pub mod role;
pub mod status;
pub mod user;

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
        .nest("/menus", menu_routes())
        .expect("static system contract")
        .nest("/roles", role_routes())
        .expect("static system contract")
        .nest("/status", status_routes())
        .expect("static system contract")
}
