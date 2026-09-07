use super::{
    admission_types::AdmissionPolicy,
    ingress::{IngestError, IngestOutcome, IngressState, MAX_BODY, ProducerKeys},
};
use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::post,
};
use chrono::Utc;
use rustzen_ipc::{
    EVENT_CREATED_HEADER, EVENT_EXPIRES_HEADER, EVENT_KEY_ID_HEADER, EVENT_NONCE_HEADER,
    EVENT_PRODUCER_HEADER, EVENT_SIGNATURE_HEADER, EVENT_VERSION_HEADER, NotificationHeaders,
};
use sqlx::SqlitePool;

pub(crate) fn router(state: IngressState) -> Router {
    Router::new().route(rustzen_ipc::NOTIFICATION_PATH, post(handler)).with_state(state)
}

pub(crate) struct IngressRuntime {
    task: tokio::task::JoinHandle<()>,
}

impl Drop for IngressRuntime {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl IngressRuntime {
    pub(crate) async fn shutdown(mut self) {
        self.task.abort();
        let _ = (&mut self.task).await;
    }
}

pub(crate) async fn start_with_keys(
    pool: SqlitePool,
    database_path: std::path::PathBuf,
    policy: AdmissionPolicy,
    address: &str,
    keys: Vec<ProducerKeys>,
) -> Result<IngressRuntime, Box<dyn std::error::Error>> {
    let socket: std::net::SocketAddr = address.parse()?;
    if !socket.ip().is_loopback() {
        return Err("notification ingress must bind loopback".into());
    }
    let state = IngressState::new_with_keys(pool, database_path, policy, keys).await?;
    let listener = tokio::net::TcpListener::bind(socket).await?;
    let task = tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, router(state)).await {
            tracing::error!(%error, "Notification ingress stopped");
        }
    });
    Ok(IngressRuntime { task })
}

async fn handler(
    State(state): State<IngressState>,
    headers: HeaderMap,
    request: axum::extract::Request,
) -> Response {
    let result = async {
        if headers.get(axum::http::header::CONTENT_TYPE).and_then(|value| value.to_str().ok())
            != Some(rustzen_ipc::NOTIFICATION_CONTENT_TYPE)
            || headers.contains_key(axum::http::header::CONTENT_ENCODING)
        {
            return Err(IngestError::BadRequest);
        }
        let signed = parse_headers(&headers)?;
        let body = axum::body::to_bytes(request.into_body(), MAX_BODY)
            .await
            .map_err(|_| IngestError::BadRequest)?;
        state.ingest(signed, &body, Utc::now()).await
    }
    .await;
    response(result)
}

fn response(result: Result<IngestOutcome, IngestError>) -> Response {
    let (status, code) = match result {
        Ok(IngestOutcome::Stored) => (StatusCode::CREATED, "stored"),
        Ok(IngestOutcome::Duplicate) => (StatusCode::OK, "duplicate"),
        Ok(IngestOutcome::NoRecipients) => (StatusCode::OK, "no-recipients"),
        Err(IngestError::BadRequest) => (StatusCode::BAD_REQUEST, "invalid-protocol"),
        Err(IngestError::Unauthorized) => (StatusCode::UNAUTHORIZED, "bad-producer"),
        Err(IngestError::Forbidden) => (StatusCode::FORBIDDEN, "forbidden-topic"),
        Err(IngestError::Expired) => (StatusCode::GONE, "event-expired"),
        Err(IngestError::RateLimited) => (StatusCode::TOO_MANY_REQUESTS, "rate-limited"),
        Err(IngestError::Conflict) => (StatusCode::CONFLICT, "event-conflict"),
        Err(IngestError::Unprocessable) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid-event"),
        Err(IngestError::Capacity) => (StatusCode::SERVICE_UNAVAILABLE, "notification-capacity"),
        Err(IngestError::Internal) => (StatusCode::INTERNAL_SERVER_ERROR, "internal-error"),
    };
    let mut response =
        Response::builder().status(status).header("content-type", "application/json");
    if matches!(status, StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE) {
        response = response.header(axum::http::header::RETRY_AFTER, "30");
    }
    response
        .body(Body::from(serde_json::json!({"code":code}).to_string()))
        .expect("static response")
}

fn parse_headers(headers: &HeaderMap) -> Result<NotificationHeaders, IngestError> {
    let one = |name: &'static str| {
        let mut values = headers.get_all(name).iter();
        let value = values.next().and_then(|v| v.to_str().ok()).ok_or(IngestError::BadRequest)?;
        if values.next().is_some() {
            return Err(IngestError::BadRequest);
        }
        Ok(value.to_string())
    };
    Ok(NotificationHeaders {
        version: one(EVENT_VERSION_HEADER)?,
        key_id: one(EVENT_KEY_ID_HEADER)?,
        producer: one(EVENT_PRODUCER_HEADER)?,
        created: one(EVENT_CREATED_HEADER)?.parse().map_err(|_| IngestError::BadRequest)?,
        expires: one(EVENT_EXPIRES_HEADER)?.parse().map_err(|_| IngestError::BadRequest)?,
        nonce: one(EVENT_NONCE_HEADER)?,
        signature: one(EVENT_SIGNATURE_HEADER)?,
    })
}

#[cfg(test)]
mod tests {
    use super::{IngestError, response};
    use axum::http::{StatusCode, header::RETRY_AFTER};

    #[test]
    fn retryable_refusals_include_retry_after() {
        for (error, status) in [
            (IngestError::RateLimited, StatusCode::TOO_MANY_REQUESTS),
            (IngestError::Capacity, StatusCode::SERVICE_UNAVAILABLE),
        ] {
            let response = response(Err(error));
            assert_eq!(response.status(), status);
            assert_eq!(response.headers()[RETRY_AFTER], "30");
        }
    }
}
