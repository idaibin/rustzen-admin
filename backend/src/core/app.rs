use crate::{
    core::db::{create_default_pool, test_connection},
    features::{
        auth::{
            middleware::auth_middleware,
            routes::{protected_auth_routes, public_auth_routes},
        },
        system::system_routes,
    },
};
use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    middleware,
    response::Json,
    routing::get,
};
use serde_json::json;
use tower_http::cors::CorsLayer;
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
#[tracing::instrument(name = "create_server")]
pub async fn create_server() -> Result<(), Box<dyn std::error::Error>> {
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
    let public_api = Router::new().nest("/auth", public_auth_routes());

    let protected_api = Router::new()
        .nest("/auth", protected_auth_routes())
        .nest("/system", system_routes())
        .route_layer(middleware::from_fn(auth_middleware));

    // Combine all routes into the final application
    let app = Router::new()
        .route("/", get(root))
        .nest("/api", public_api.merge(protected_api))
        .layer(cors)
        .with_state(pool);

    // Start the server
    let addr = get_addr().await;
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("🚀 Server started successfully, listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Retrieves the server's listening address from environment variables.
///
/// Defaults to `0.0.0.0:8000` if `APP_HOST` or `APP_PORT` are not set.
async fn get_addr() -> String {
    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "8000".to_string());
    tracing::debug!("Server configured to run on {}:{}", host, port);
    format!("{}:{}", host, port)
}

/// Handles requests to the root (`/`) endpoint.
///
/// Provides a simple welcome message and API version information.
async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "message": "Welcome to rustzen-admin API",
        "version": "0.1.0",
        "description": "A backend management system built with Rust, Axum, SQLx, and PostgreSQL."
    }))
}
