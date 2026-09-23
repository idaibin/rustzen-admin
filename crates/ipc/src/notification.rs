use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write;

const DOMAIN: &str = "rz-notification-producer-v1";
const VERSION: &str = "1";
type HmacSha256 = Hmac<Sha256>;

pub const NOTIFICATION_PATH: &str = "/internal/v1/notification-events";
pub const NOTIFICATION_METHOD: &str = "POST";
pub const NOTIFICATION_CONTENT_TYPE: &str = "application/json";
pub const EVENT_VERSION_HEADER: &str = "x-rustzen-notify-version";
pub const EVENT_KEY_ID_HEADER: &str = "x-rustzen-notify-key-id";
pub const EVENT_PRODUCER_HEADER: &str = "x-rustzen-notify-producer";
pub const EVENT_CREATED_HEADER: &str = "x-rustzen-notify-created";
pub const EVENT_EXPIRES_HEADER: &str = "x-rustzen-notify-expires";
pub const EVENT_NONCE_HEADER: &str = "x-rustzen-notify-nonce";
pub const EVENT_SIGNATURE_HEADER: &str = "x-rustzen-notify-signature";

pub fn valid_notification_key_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationEvent {
    pub schema_version: u8,
    pub event_id: String,
    pub producer: String,
    pub topic: String,
    pub occurred_at: String,
    pub expires_at: String,
    pub subject: NotificationSubject,
    pub audience: NotificationAudience,
    pub content: NotificationContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationSubject {
    pub kind: String,
    pub id: String,
    pub revision: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationAudience {
    pub policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiator_user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationContent {
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NotificationHeaders {
    pub version: String,
    pub key_id: String,
    pub producer: String,
    pub created: i64,
    pub expires: i64,
    pub nonce: String,
    pub signature: String,
}

pub struct NotificationSigner {
    key_id: String,
    producer: String,
    secret: Vec<u8>,
}

impl NotificationSigner {
    pub fn new(
        key_id: impl Into<String>,
        producer: impl Into<String>,
        secret: &[u8],
    ) -> Result<Self, NotificationAuthError> {
        let key_id = key_id.into();
        let producer = producer.into();
        if !valid_notification_key_id(&key_id) || producer.is_empty() || secret.len() < 32 {
            return Err(NotificationAuthError::InvalidConfiguration);
        }
        Ok(Self { key_id, producer, secret: secret.to_vec() })
    }

    pub fn sign(
        &self,
        body: &[u8],
        created: i64,
        expires: i64,
        nonce: impl Into<String>,
    ) -> Result<NotificationHeaders, NotificationAuthError> {
        let nonce = nonce.into();
        if nonce.is_empty() || expires <= created {
            return Err(NotificationAuthError::InvalidEnvelope);
        }
        let message = canonical(&self.key_id, &self.producer, created, expires, &nonce, body);
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .map_err(|_| NotificationAuthError::InvalidConfiguration)?;
        mac.update(&message);
        Ok(NotificationHeaders {
            version: VERSION.into(),
            key_id: self.key_id.clone(),
            producer: self.producer.clone(),
            created,
            expires,
            nonce,
            signature: hex::encode(mac.finalize().into_bytes()),
        })
    }
}

pub fn verify_notification(
    headers: &NotificationHeaders,
    body: &[u8],
    expected_key_id: &str,
    secret: &[u8],
) -> Result<(), NotificationAuthError> {
    if headers.version != VERSION
        || !valid_notification_key_id(&headers.key_id)
        || headers.key_id != expected_key_id
        || headers.nonce.is_empty()
        || headers.expires <= headers.created
        || headers.signature.len() != 64
    {
        return Err(NotificationAuthError::InvalidEnvelope);
    }
    let signature =
        hex::decode(&headers.signature).map_err(|_| NotificationAuthError::InvalidEnvelope)?;
    let message = canonical(
        &headers.key_id,
        &headers.producer,
        headers.created,
        headers.expires,
        &headers.nonce,
        body,
    );
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| NotificationAuthError::InvalidConfiguration)?;
    mac.update(&message);
    mac.verify_slice(&signature).map_err(|_| NotificationAuthError::InvalidSignature)
}

fn canonical(
    key_id: &str,
    producer: &str,
    created: i64,
    expires: i64,
    nonce: &str,
    body: &[u8],
) -> Vec<u8> {
    let body_hash = hex::encode(Sha256::digest(body));
    let fields = [
        DOMAIN.to_string(),
        VERSION.to_string(),
        key_id.to_string(),
        producer.to_string(),
        "admin".to_string(),
        NOTIFICATION_METHOD.to_string(),
        NOTIFICATION_PATH.to_string(),
        NOTIFICATION_CONTENT_TYPE.to_string(),
        body_hash,
        created.to_string(),
        expires.to_string(),
        nonce.to_string(),
    ];
    let mut output = Vec::new();
    for field in fields {
        let mut length = String::new();
        write!(&mut length, "{}:", field.len()).expect("write canonical length");
        output.extend_from_slice(length.as_bytes());
        output.extend_from_slice(field.as_bytes());
    }
    output
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum NotificationAuthError {
    #[error("invalid notification transport configuration")]
    InvalidConfiguration,
    #[error("invalid notification transport envelope")]
    InvalidEnvelope,
    #[error("invalid notification transport signature")]
    InvalidSignature,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_binds_body_identity_time_nonce_and_target() {
        let secret = ["0123456789abcdef", "0123456789abcdef"].concat();
        let nonce = ["nonce", "-1"].concat();
        let signer = NotificationSigner::new("key-1", "monitor", secret.as_bytes()).unwrap();
        let headers = signer.sign(b"{}", 10, 20, &nonce).unwrap();
        assert!(verify_notification(&headers, b"{}", "key-1", secret.as_bytes()).is_ok());
        assert_eq!(
            verify_notification(&headers, b"{ }", "key-1", secret.as_bytes()),
            Err(NotificationAuthError::InvalidSignature)
        );
        assert_eq!(
            verify_notification(&headers, b"{}", "key-2", secret.as_bytes()),
            Err(NotificationAuthError::InvalidEnvelope)
        );
    }

    #[test]
    fn signature_matches_independent_frozen_golden_vector() {
        let secret = ["0123456789abcdef", "0123456789abcdef"].concat();
        let nonce = ["a", "bc"].concat();
        let signer = NotificationSigner::new("key-1", "monitor", secret.as_bytes()).unwrap();
        let headers = signer.sign(br#"{"x":1}"#, 1_700_000_000, 1_700_000_060, &nonce).unwrap();
        assert_eq!(
            headers.signature,
            "4af5d59a23f3d472ed96b7fb4c3839f7e8cf5b06ff83b82c036ea5ebde13f850"
        );
    }

    #[test]
    fn key_id_grammar_is_safe_at_the_http_header_boundary() {
        let valid = format!("A._-{}", "z".repeat(60));
        assert_eq!(valid.len(), 64);
        assert!(valid_notification_key_id(&valid));
        assert!(http::HeaderValue::from_str(&valid).is_ok());
        for invalid in
            ["".into(), "key id".into(), "key\nforged".into(), "key:colon".into(), "x".repeat(65)]
        {
            assert!(!valid_notification_key_id(&invalid), "accepted {invalid:?}");
            assert!(
                NotificationSigner::new(&invalid, "monitor", b"0123456789abcdef0123456789abcdef",)
                    .is_err()
            );
        }
    }
}
