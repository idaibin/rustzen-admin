use sha2::{Digest, Sha256};

use crate::delegation::{
    CONTRACT_VERSION, DELEGATION_TTL, IPC_ACCESS_HEADER, IPC_CONTRACT_VERSION_HEADER,
    IPC_MODULE_HEADER, IPC_REQUEST_ID_HEADER, IPC_SIGNATURE_HEADER, IPC_TIMESTAMP_HEADER,
    IPC_USER_ID_HEADER,
};

/// Canonical wire descriptor for the existing signed delegation boundary.
pub fn canonical_delegation_descriptor() -> String {
    serde_json::json!({
        "access": {
            "protected": "userId > 0 and capability is <module>:<lowercase-alnum-_- segments>",
            "public": "userId is anonymous"
        },
        "framing": [
            "contractVersion", "timestamp", "requestId", "userIdOrAnonymous",
            "module", "method", "path", "capability"
        ],
        "headers": {
            "access": IPC_ACCESS_HEADER.as_str(),
            "contractVersion": IPC_CONTRACT_VERSION_HEADER.as_str(),
            "module": IPC_MODULE_HEADER.as_str(),
            "requestId": IPC_REQUEST_ID_HEADER.as_str(),
            "signature": IPC_SIGNATURE_HEADER.as_str(),
            "timestamp": IPC_TIMESTAMP_HEADER.as_str(),
            "userId": IPC_USER_ID_HEADER.as_str()
        },
        "signature": {"algorithm": "hmac-sha256", "encoding": "lowercase-hex"},
        "ttlSeconds": DELEGATION_TTL.as_secs(),
        "version": CONTRACT_VERSION
    })
    .to_string()
}

pub fn delegation_protocol_digest() -> String {
    format!("{:x}", Sha256::digest(canonical_delegation_descriptor().as_bytes()))
}

pub fn delegation_protocol_output() -> String {
    format!("{}\n{}", canonical_delegation_descriptor(), delegation_protocol_digest())
}

#[cfg(test)]
mod tests {
    use super::{canonical_delegation_descriptor, delegation_protocol_digest};

    #[test]
    fn descriptor_binds_delegation_headers_framing_and_rules() {
        let descriptor = canonical_delegation_descriptor();
        assert!(descriptor.contains("x-rustzen-ipc-signature"));
        assert!(descriptor.contains("userIdOrAnonymous"));
        assert!(descriptor.contains("hmac-sha256"));
        assert_eq!(delegation_protocol_digest().len(), 64);
    }
}
