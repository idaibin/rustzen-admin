use crate::features::system::log::{service::LogService, types::LogWriteCommand};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::Method,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use rustzen_core::auth::CurrentUser;
use sqlx::PgPool;
use std::{net::SocketAddr, time::Instant};

fn should_log(method: &Method, path: &str) -> bool {
    !matches!(
        (method, path),
        (&Method::GET, "/api/dashboard/health")
            | (&Method::GET, "/api/dashboard/metrics")
            | (&Method::GET, "/api/dashboard/stats")
            | (&Method::GET, "/api/dashboard/trends")
    )
}

/// HTTP logging middleware.
pub async fn log_middleware(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let uri = request.uri().to_string();
    let user_agent = request_user_agent(&request);
    let client_ip = addr.ip().to_string();
    tracing::debug!(method = %method, uri = %uri, client_ip = %client_ip, "Handling request");
    let current_user = request.extensions().get::<CurrentUser>().cloned();
    let response = next.run(request).await;
    let duration = start.elapsed();

    let user_id = current_user.as_ref().map(|u| u.user_id);
    let username = current_user.as_ref().map(|u| u.username.as_str()).unwrap_or("anonymous");
    let status_code = response.status().as_u16();
    let method_for_log = method.clone();

    if should_log(&method, &path) {
        if let Err(e) = LogService::record_operation(
            &pool,
            build_request_log(
                user_id.unwrap_or(0),
                username,
                method_for_log,
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
        } else {
            tracing::debug!(
                method = %method,
                uri = %uri,
                status_code,
                duration_ms = duration.as_millis(),
                user_id = user_id.unwrap_or(0),
                "HTTP request recorded"
            );
        }
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

#[cfg(test)]
mod tests {
    use super::should_log;
    use axum::http::Method;

    #[test]
    fn should_log_skips_dashboard_read_endpoints() {
        assert!(!should_log(&Method::GET, "/api/dashboard/health"));
        assert!(!should_log(&Method::GET, "/api/dashboard/metrics"));
        assert!(!should_log(&Method::GET, "/api/dashboard/stats"));
        assert!(!should_log(&Method::GET, "/api/dashboard/trends"));
    }

    #[test]
    fn should_log_keeps_business_and_non_get_requests() {
        assert!(should_log(&Method::POST, "/api/dashboard/health"));
        assert!(should_log(&Method::GET, "/api/system/user"));
    }
}
