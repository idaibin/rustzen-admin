use crate::{
    common::api::{ApiResponse, AppResult},
    features::{
        auth::{protected_auth_routes, public_auth_routes},
        dashboard::dashboard_routes,
        system::system_routes,
    },
    infra::{
        config::CONFIG,
        db::{create_default_pool, test_connection},
    },
    middleware::{auth::auth_middleware, log::log_middleware},
};

use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    middleware,
    routing::get,
};
use serde_json::json;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::{cors::CorsLayer, services::{ServeDir, ServeFile}};

#[tracing::instrument(name = "run_server")]
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database connection pool...");
    let pool = create_default_pool().await?;
    test_connection(&pool).await?;

    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("*"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT]);

    let protected_api = Router::new()
        .nest("/auth", protected_auth_routes())
        .nest("/dashboard", dashboard_routes())
        .nest("/system", system_routes())
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware))
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware));

    let public_api = Router::new().nest("/auth", public_auth_routes());
    let uploads_prefix = CONFIG.upload_public_prefix.clone();
    let uploads_service =
        ServeDir::new(&CONFIG.upload_dir).append_index_html_on_directories(true);
    let static_dir = PathBuf::from(&CONFIG.web_dist);
    let index_path = static_dir.join("index.html");

    tracing::info!(?static_dir, "Serving frontend assets from static dir");

    let app = Router::new()
        .route("/api/summary", get(summary))
        .nest("/api", public_api.merge(protected_api))
        .nest_service(&uploads_prefix, uploads_service)
        .layer(cors)
        .with_state(pool)
        .fallback_service(ServeDir::new(static_dir).not_found_service(ServeFile::new(index_path)))
        .into_make_service_with_connect_info::<SocketAddr>();

    let addr = server_addr();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server started successfully, listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn server_addr() -> String {
    format!("{}:{}", CONFIG.app_host, CONFIG.app_port)
}

async fn summary() -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(json!({
        "message": "Welcome to rustzen-admin API",
        "description": "A backend management system built with Rust, Axum, SQLx, and PostgreSQL.",
        "github": "https://github.com/idaibin/rustzen-admin"
    })))
}
