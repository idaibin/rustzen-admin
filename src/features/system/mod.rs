pub mod dict;
pub mod log;
pub mod menu;
pub mod role;
pub mod user;

use axum::Router;
use sqlx::PgPool;

use dict::api::dict_routes;
use log::api::log_routes;
use menu::api::menu_routes;
use role::api::role_routes;
use user::api::user_routes;

/// 系统路由
pub fn system_routes() -> Router<PgPool> {
    Router::new()
        .nest("/users", user_routes())
        .nest("/menus", menu_routes())
        .nest("/roles", role_routes())
        .nest("/dicts", dict_routes())
        .nest("/logs", log_routes())
}
