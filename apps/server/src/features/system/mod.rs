pub mod menu;
pub mod role;
pub mod user;

use axum::Router;
use sqlx::SqlitePool;

use menu::menu_routes;
use role::role_routes;
use user::user_routes;

pub fn system_routes() -> Router<SqlitePool> {
    Router::new()
        .nest("/users", user_routes())
        .nest("/menus", menu_routes())
        .nest("/roles", role_routes())
}
