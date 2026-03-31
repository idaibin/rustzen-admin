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
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing;

/// Creates and starts the main application server.
///
/// This function orchestrates the entire application startup process:
/// 1. Initializes the database connection pool.
/// 2. Sets up Cross-Origin Resource Sharing (CORS) middleware.
/// 3. Defines and separates public and protected API routes.
/// 4. Applies authentication middleware to protected routes.
/// 5. Binds to a TCP listener and serves the application.
///
/// # Errors
///
/// Returns an error if any part of the setup fails, such as database
/// connection, or binding to the network address.
#[tracing::instrument(name = "run_server")]
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database pool
    tracing::info!("Initializing database connection pool...");
    let pool = create_default_pool().await?;
    test_connection(&pool).await?;

    // Configure CORS
    tracing::info!("Configuring CORS middleware...");
    // DEV-NOTE: This is a permissive CORS configuration for development.
    // For production, restrict origins to the actual frontend URL.
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION, ACCEPT]);

    // Define public and protected routes
    tracing::info!("Setting up API routes...");
    let protected_api = Router::new()
        .nest("/auth", protected_auth_routes())
        .nest("/dashboard", dashboard_routes())
        .nest("/system", system_routes())
        .route_layer(middleware::from_fn_with_state(pool.clone(), log_middleware)) // log middleware
        .route_layer(middleware::from_fn_with_state(pool.clone(), auth_middleware)); // auth middleware

    // public routes
    let public_api = Router::new().nest("/auth", public_auth_routes());

    // uploads file service
    let uploads_service = ServeDir::new("uploads")
        .not_found_service(ServeDir::new("uploads").append_index_html_on_directories(true));

    // combine all routes
    let app = Router::new()
        .route("/api/summary", get(summary))
        .nest("/api", public_api.merge(protected_api))
        .nest_service("/uploads", uploads_service) // uploads file service
        .layer(cors)
        .with_state(pool)
        /*
         * tips: if you want to use web embed
         * 1. you need to cancel comment in the web_embed module
         * 2. you need to cancel comment next line
         */
        // .fallback(crate::infra::web_embed::web_embed_file_handler)
        .fallback_service(ServeDir::new("web/dist").append_index_html_on_directories(true))
        .into_make_service_with_connect_info::<SocketAddr>();

    // get server address
    let addr = server_addr().await;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Server started successfully, listening on http://{}", addr);

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

/// Retrieves the server's listening address from environment variables.
///
/// Defaults to `0.0.0.0:8000` if `APP_HOST` or `APP_PORT` are not set.
async fn server_addr() -> String {
    let addr = format!("{}:{}", CONFIG.app_host, CONFIG.app_port);
    tracing::debug!("Server configured to run on {}", addr);
    addr
}

/// Handles requests to the root (`/`) endpoint.
///
/// Provides a simple welcome message and API version information.
async fn summary() -> AppResult<serde_json::Value> {
    Ok(ApiResponse::success(json!({
        "message": "Welcome to rustzen-admin API",
        "description": "A backend management system built with Rust, Axum, SQLx, and PostgreSQL.",
        "github": "https://github.com/idaibin/rustzen-admin"
    })))
}
