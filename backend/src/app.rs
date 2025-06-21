use crate::features;
use axum::Router;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

/// 构建 Axum 应用
pub async fn create_app() -> Router {
    // 1. 创建 /sys 路由，并挂载所有业务模块
    let sys_router = Router::new()
        .nest("/user", features::user::router())
        .nest("/role", features::role::router())
        .nest("/menu", features::menu::router())
        .nest("/dict", features::dict::router())
        .nest("/log", features::log::router());

    // 2. 将 /sys 路由挂载到主路由的 /api 下
    let api_router = Router::new().nest("/sys", sys_router);

    // 3. 创建应用根路由，并应用中间件
    Router::new()
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::very_permissive())
}
