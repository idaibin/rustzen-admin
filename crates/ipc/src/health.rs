use serde::{Deserialize, Serialize};

use crate::CONTRACT_VERSION;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub contract_version: u32,
    pub release_version: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_binding: Option<SelectedBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedBinding {
    pub build_id: String,
    pub composition_id: String,
}

impl HealthResponse {
    pub fn ok(release_version: impl Into<String>) -> Self {
        Self {
            contract_version: CONTRACT_VERSION,
            release_version: release_version.into(),
            status: "ok".to_owned(),
            selected_binding: None,
        }
    }
    pub fn ok_selected(release_version: impl Into<String>) -> Self {
        let mut response = Self::ok(release_version);
        response.selected_binding =
            match (std::env::var("RUSTZEN_BUILD_ID"), std::env::var("RUSTZEN_COMPOSITION_ID")) {
                (Ok(build_id), Ok(composition_id))
                    if is_sha256(&build_id) && is_sha256(&composition_id) =>
                {
                    Some(SelectedBinding { build_id, composition_id })
                }
                _ => None,
            };
        response
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::{HealthResponse, is_sha256};

    #[test]
    fn ok_serializes_and_deserializes_the_health_contract() {
        let response = HealthResponse::ok("0.5.0");
        let body = serde_json::to_string(&response).expect("serialize health response");

        assert_eq!(body, r#"{"contractVersion":1,"releaseVersion":"0.5.0","status":"ok"}"#);
        assert_eq!(
            serde_json::from_str::<HealthResponse>(&body).expect("deserialize health response"),
            response
        );
    }

    #[test]
    fn selected_binding_accepts_only_lowercase_sha256_values() {
        assert!(is_sha256(&"0".repeat(64)));
        assert!(!is_sha256(&"A".repeat(64)));
        assert!(!is_sha256(&"g".repeat(64)));
    }
}
