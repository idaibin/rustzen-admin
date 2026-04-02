use crate::{
    features::system::log::{service::LogService, types::LogWriteCommand},
    infra::extractor::CurrentUser,
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use std::{net::SocketAddr, time::Instant};

/// HTTP logging middleware.
pub async fn log_middleware(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().to_string();
    let user_agent = request_user_agent(&request);
    let client_ip = addr.ip().to_string();
    let current_user = request.extensions().get::<CurrentUser>().cloned();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let user_id = current_user.as_ref().map(|u| u.user_id);
    let username = current_user.as_ref().map(|u| u.username.as_str()).unwrap_or("anonymous");
    let status_code = response.status().as_u16();

    if let Err(e) = LogService::record_operation(
        &pool,
        build_request_log(
            user_id.unwrap_or(0),
            username,
            method,
            &uri,
            status_code,
            duration,
            client_ip,
            user_agent,
        ),
    )
    .await
    {
        tracing::error!("Failed to log HTTP request: {:?}", e);
    }

    Ok(response)
}

fn request_user_agent(request: &Request) -> String {
    request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string()
}

fn build_request_log(
    user_id: i64,
    username: &str,
    method: axum::http::Method,
    uri: &str,
    status_code: u16,
    duration: std::time::Duration,
    ip_address: String,
    user_agent: String,
) -> LogWriteCommand {
    let status = if status_code < 400 { "SUCCESS" } else { "ERROR" };
    LogWriteCommand {
        user_id,
        username: username.to_string(),
        action: format!("HTTP_{}", method),
        description: format!("{} {} - {}", method, uri, status_code),
        data: None,
        status: status.to_string(),
        duration_ms: duration.as_millis() as i32,
        ip_address,
        user_agent,
    }
}
