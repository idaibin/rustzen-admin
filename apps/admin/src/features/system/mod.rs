#[cfg(feature = "selected-distribution")]
pub mod access;
#[cfg(feature = "full")]
pub mod menu;
pub mod role;
#[cfg(feature = "full")]
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
    #[cfg(feature = "full")]
    let router = router
        .nest("/menus", menu::menu_routes())
        .expect("static system contract")
        .nest("/status", status::status_routes())
        .expect("static system contract");
    #[cfg(feature = "selected-distribution")]
    let router = router
        .nest("/menus", access::permission_option_routes())
        .expect("static monitor access contract");
    router
}
