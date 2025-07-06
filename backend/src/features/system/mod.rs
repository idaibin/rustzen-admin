use axum::Router;
use sqlx::PgPool;

pub mod dict;
pub mod log;
pub mod menu;
pub mod role;
pub mod user;

use dict::router::dict_routes;
use log::router::log_routes;
use menu::router::menu_routes;
use role::router::role_routes;
use user::router::user_routes;

/// 系统路由
pub fn system_routes() -> Router<PgPool> {
    Router::new()
        .nest("/users", user_routes())
        .nest("/menus", menu_routes())
        .nest("/roles", role_routes())
        .nest("/dicts", dict_routes())
        .nest("/logs", log_routes())
}
