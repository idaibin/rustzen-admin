use crate::common::error::ServiceError;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const TOKEN_VERSION: u8 = 1;
const AAD: &[u8] = b"rustzen-admin-notification-cursor-v1";

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum CursorKind {
    Page,
    Snapshot,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct CursorState {
    pub version: u8,
    pub kind: CursorKind,
    pub user_id: i64,
    pub unread_only: bool,
    pub max_seq: i64,
    pub before_seq: Option<i64>,
}

pub(super) fn encode(state: &CursorState, secret: &[u8]) -> Result<String, ServiceError> {
    let payload = serde_json::to_vec(state).map_err(|_| invalid_cursor())?;
    let cipher = cipher(secret)?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, Payload { msg: &payload, aad: AAD })
        .map_err(|_| invalid_cursor())?;
    let mut token = Vec::with_capacity(1 + nonce.len() + ciphertext.len());
    token.push(TOKEN_VERSION);
    token.extend_from_slice(&nonce);
    token.extend_from_slice(&ciphertext);
    Ok(URL_SAFE_NO_PAD.encode(token))
}

pub(super) fn decode(token: &str, secret: &[u8]) -> Result<CursorState, ServiceError> {
    let token = URL_SAFE_NO_PAD.decode(token).map_err(|_| invalid_cursor())?;
    if token.len() <= 1 + 12 || token[0] != TOKEN_VERSION {
        return Err(invalid_cursor());
    }
    let nonce = Nonce::from(<[u8; 12]>::try_from(&token[1..13]).map_err(|_| invalid_cursor())?);
    let payload = cipher(secret)?
        .decrypt(&nonce, Payload { msg: &token[13..], aad: AAD })
        .map_err(|_| invalid_cursor())?;
    let state: CursorState = serde_json::from_slice(&payload).map_err(|_| invalid_cursor())?;
    if state.version != 1 || state.user_id <= 0 || state.max_seq < 0 {
        return Err(invalid_cursor());
    }
    match state.kind {
        CursorKind::Page if state.before_seq.is_some_and(|value| value > 0) => Ok(state),
        CursorKind::Snapshot if state.before_seq.is_none() => Ok(state),
        _ => Err(invalid_cursor()),
    }
}

fn cipher(secret: &[u8]) -> Result<Aes256Gcm, ServiceError> {
    if secret.is_empty() {
        return Err(invalid_cursor());
    }
    let mut digest = Sha256::new();
    digest.update(AAD);
    digest.update(secret);
    let key = digest.finalize();
    Aes256Gcm::new_from_slice(&key).map_err(|_| invalid_cursor())
}

fn invalid_cursor() -> ServiceError {
    ServiceError::InvalidOperation("Invalid inbox cursor or snapshot".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_is_encrypted_authenticated_and_bound_to_its_full_state() {
        let state = CursorState {
            version: 1,
            kind: CursorKind::Page,
            user_id: 7,
            unread_only: true,
            max_seq: 42,
            before_seq: Some(21),
        };
        let token = encode(&state, b"test-secret").unwrap();
        let external = URL_SAFE_NO_PAD.decode(&token).unwrap();
        let plaintext = serde_json::to_vec(&state).unwrap();
        assert_eq!(external[0], TOKEN_VERSION);
        assert_ne!(&external[13..], plaintext);
        assert!(!token.contains('{') && !token.contains('"'));
        assert_eq!(decode(&token, b"test-secret").unwrap(), state);
        assert!(decode(&token, b"other-secret").is_err());
        let mut bytes = token.into_bytes();
        bytes[3] = if bytes[3] == b'a' { b'b' } else { b'a' };
        assert!(decode(std::str::from_utf8(&bytes).unwrap(), b"test-secret").is_err());
    }
}
