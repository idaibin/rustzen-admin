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

/// HTTP 日志中间件：自动记录写操作和错误请求
pub async fn log_middleware(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    let start = Instant::now();

    // 提取请求信息
    let method = request.method().clone();
    let uri = request.uri().to_string();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();
    let client_ip = addr.ip().to_string();

    // 获取当前用户
    let current_user = request.extensions().get::<CurrentUser>().cloned();

    // 处理请求
    let response = next.run(request).await;
    let duration = start.elapsed();

    let user_id = current_user.as_ref().map(|u| u.user_id);
    let username = current_user.as_ref().map(|u| u.username.as_str());
    let status_code = response.status().as_u16();

    let action = format!("HTTP_{}", method);
    let status = if status_code < 400 { "SUCCESS" } else { "ERROR" };
    let description = format!("{} {} - {}", method, uri, status_code);

    if let Err(e) = LogService::record_operation(
        &pool,
        LogWriteCommand {
            user_id: user_id.unwrap_or(0),
            username: username.unwrap_or("anonymous").to_string(),
            action,
            description,
            data: None,
            status: status.to_string(),
            duration_ms: duration.as_millis() as i32,
            ip_address: client_ip,
            user_agent,
        },
    )
    .await
    {
        tracing::error!("Failed to log HTTP request: {:?}", e);
    }

    Ok(response)
}
