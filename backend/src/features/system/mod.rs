use axum::Router;
use sqlx::PgPool;

pub mod dict;
pub mod log;
pub mod menu;
pub mod role;
pub mod user;

use dict::routes::dict_routes;
use log::routes::log_routes;
use menu::routes::menu_routes;
use role::routes::role_routes;
use user::routes::user_routes;

/// 系统路由
pub fn system_routes() -> Router<PgPool> {
    Router::new()
        .nest("/users", user_routes())
        .nest("/menus", menu_routes())
        .nest("/roles", role_routes())
        .nest("/dicts", dict_routes())
        .nest("/logs", log_routes())
}
