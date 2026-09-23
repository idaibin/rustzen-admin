use super::relay::{Claim, DeliveryResult, Transport};
use rustzen_ipc::{
    EVENT_CREATED_HEADER, EVENT_EXPIRES_HEADER, EVENT_KEY_ID_HEADER, EVENT_NONCE_HEADER,
    EVENT_PRODUCER_HEADER, EVENT_SIGNATURE_HEADER, EVENT_VERSION_HEADER, NotificationSigner,
};
use sha2::{Digest, Sha256};
use std::{future::Future, pin::Pin, time::Duration};
use uuid::Uuid;

pub(crate) struct HttpTransport {
    client: reqwest::Client,
    url: String,
    signer: NotificationSigner,
}

impl HttpTransport {
    pub(crate) fn new(
        url: String,
        key_id: String,
        secret: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_timeout(url, key_id, secret, Duration::from_secs(5))
    }

    fn with_timeout(
        url: String,
        key_id: String,
        secret: &[u8],
        timeout: Duration,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client: reqwest::Client::builder()
                .timeout(timeout)
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
            url,
            signer: NotificationSigner::new(key_id, "monitor", secret)?,
        })
    }
}

impl Transport for HttpTransport {
    fn send<'a>(
        &'a self,
        claim: &'a Claim,
    ) -> Pin<Box<dyn Future<Output = DeliveryResult> + Send + 'a>> {
        Box::pin(async move {
            if hex::encode(Sha256::digest(claim.payload_json.as_bytes())) != claim.payload_sha256 {
                return DeliveryResult::Invalid;
            }
            let now = chrono::Utc::now().timestamp();
            let Ok(headers) = self.signer.sign(
                claim.payload_json.as_bytes(),
                now,
                now + 60,
                Uuid::new_v4().to_string(),
            ) else {
                return DeliveryResult::Unauthorized;
            };
            let response = self
                .client
                .post(&self.url)
                .header("content-type", rustzen_ipc::NOTIFICATION_CONTENT_TYPE)
                .header(EVENT_VERSION_HEADER, headers.version)
                .header(EVENT_KEY_ID_HEADER, headers.key_id)
                .header(EVENT_PRODUCER_HEADER, headers.producer)
                .header(EVENT_CREATED_HEADER, headers.created)
                .header(EVENT_EXPIRES_HEADER, headers.expires)
                .header(EVENT_NONCE_HEADER, headers.nonce)
                .header(EVENT_SIGNATURE_HEADER, headers.signature)
                .body(claim.payload_json.clone())
                .send()
                .await;
            let response = match response {
                Ok(response) => response,
                Err(error) if error.is_builder() => return DeliveryResult::Invalid,
                Err(error) if error.is_connect() => return DeliveryResult::Unavailable,
                Err(_) => return DeliveryResult::Timeout,
            };
            let status = response.status();
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<i64>().ok())
                .filter(|seconds| (1..=60).contains(seconds));
            let bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => return DeliveryResult::Timeout,
            };
            let code = serde_json::from_slice::<serde_json::Value>(&bytes)
                .ok()
                .and_then(|value| value.get("code")?.as_str().map(str::to_owned));
            classify(status.as_u16(), code.as_deref(), retry_after)
        })
    }
}

fn classify(status: u16, code: Option<&str>, retry_after: Option<i64>) -> DeliveryResult {
    match (status, code, retry_after) {
        (201, Some("stored"), _) => DeliveryResult::Stored,
        (200, Some("duplicate"), _) => DeliveryResult::Duplicate,
        (200, Some("no-recipients"), _) => DeliveryResult::NoRecipients,
        (409, _, _) => DeliveryResult::Conflict,
        (410, _, _) => DeliveryResult::Expired,
        (400 | 413 | 422, _, _) => DeliveryResult::Invalid,
        (401 | 403, _, _) => DeliveryResult::Unauthorized,
        (429, Some("rate-limited"), Some(retry_after_seconds)) => {
            DeliveryResult::RateLimited { retry_after_seconds }
        }
        (503, Some("notification-capacity"), Some(retry_after_seconds)) => {
            DeliveryResult::Capacity { retry_after_seconds }
        }
        _ => DeliveryResult::Ambiguous,
    }
}

#[cfg(test)]
mod tests {
    use super::{Claim, DeliveryResult, HttpTransport, Transport, classify};
    use sha2::{Digest, Sha256};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn response_matrix_is_exact_and_fail_closed() {
        for (status, code, retry_after, expected) in [
            (201, Some("stored"), None, DeliveryResult::Stored),
            (200, Some("duplicate"), None, DeliveryResult::Duplicate),
            (200, Some("no-recipients"), None, DeliveryResult::NoRecipients),
            (409, Some("event-conflict"), None, DeliveryResult::Conflict),
            (410, Some("event-expired"), None, DeliveryResult::Expired),
            (400, Some("invalid-protocol"), None, DeliveryResult::Invalid),
            (413, None, None, DeliveryResult::Invalid),
            (422, Some("invalid-event"), None, DeliveryResult::Invalid),
            (401, Some("bad-producer"), None, DeliveryResult::Unauthorized),
            (403, Some("forbidden-topic"), None, DeliveryResult::Unauthorized),
            (
                429,
                Some("rate-limited"),
                Some(30),
                DeliveryResult::RateLimited { retry_after_seconds: 30 },
            ),
            (
                503,
                Some("notification-capacity"),
                Some(30),
                DeliveryResult::Capacity { retry_after_seconds: 30 },
            ),
            (429, Some("rate-limited"), None, DeliveryResult::Ambiguous),
            (201, Some("changed"), None, DeliveryResult::Ambiguous),
            (500, None, None, DeliveryResult::Ambiguous),
        ] {
            assert_eq!(classify(status, code, retry_after), expected);
        }
        assert!(
            HttpTransport::new(
                "http://127.0.0.1:9811/internal/v1/notification-events".into(),
                "key\nforged".into(),
                b"0123456789abcdef0123456789abcdef",
            )
            .is_err(),
            "invalid header bytes must fail before transport"
        );
    }

    fn claim() -> Claim {
        let payload_json = r#"{"schemaVersion":1}"#.to_string();
        Claim {
            event_id: "event-1".into(),
            payload_sha256: hex::encode(Sha256::digest(payload_json.as_bytes())),
            payload_json,
            expires_at: "2026-09-08T00:00:00Z".into(),
            lease_token: "lease-1".into(),
            attempts: 1,
        }
    }

    async fn read_request(stream: &mut tokio::net::TcpStream) {
        let mut bytes = vec![0_u8; 4096];
        let read = stream.read(&mut bytes).await.unwrap();
        assert!(read > 0);
    }

    fn transport(url: String, timeout: Duration) -> HttpTransport {
        HttpTransport::with_timeout(
            url,
            "key-1".into(),
            b"0123456789abcdef0123456789abcdef",
            timeout,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn committed_peer_response_timeout_is_ambiguous() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            read_request(&mut stream).await;
            // Receipt of the complete bounded request stands for the peer commit boundary.
            tokio::time::sleep(Duration::from_millis(100)).await;
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 20\r\nconnection: close\r\n\r\n{\"code\":\"duplicate\"}")
                .await
                .ok();
        });
        let result = transport(
            format!("http://{address}/internal/v1/notification-events"),
            Duration::from_millis(20),
        )
        .send(&claim())
        .await;
        assert_eq!(result, DeliveryResult::Timeout);
        server.await.unwrap();
    }

    #[tokio::test]
    async fn redirect_is_not_followed() {
        let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target_address = target.local_addr().unwrap();
        let redirect = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let redirect_address = redirect.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = redirect.accept().await.unwrap();
            read_request(&mut stream).await;
            let response = format!(
                "HTTP/1.1 307 Temporary Redirect\r\nlocation: http://{target_address}/internal/v1/notification-events\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let result = transport(
            format!("http://{redirect_address}/internal/v1/notification-events"),
            Duration::from_secs(1),
        )
        .send(&claim())
        .await;
        assert_eq!(result, DeliveryResult::Ambiguous);
        assert!(tokio::time::timeout(Duration::from_millis(50), target.accept()).await.is_err());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn connect_before_send_failure_remains_retryable() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let result = transport(
            format!("http://{address}/internal/v1/notification-events"),
            Duration::from_secs(1),
        )
        .send(&claim())
        .await;
        assert_eq!(result, DeliveryResult::Unavailable);
    }
}
