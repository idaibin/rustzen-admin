use std::sync::Arc;

use chrono::{DateTime, Utc};
use rustzen_ipc::{AccessMode, ModuleManifest};
use serde::{Deserialize, Serialize};

use crate::infra::config::CONFIG;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModuleSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub base_url: String,
}

impl ModuleSpec {
    pub fn fixed() -> Vec<Self> {
        vec![
            Self { id: "monitor", name: "监控", base_url: CONFIG.monitor_base_url() },
            Self { id: "insights", name: "分析", base_url: CONFIG.insights_base_url() },
            Self { id: "reports", name: "报表", base_url: CONFIG.reports_base_url() },
        ]
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModuleCondition {
    Unavailable,
    Healthy,
    Incompatible,
}

#[derive(Debug, Clone)]
pub struct ModuleRuntime {
    pub spec: ModuleSpec,
    pub enabled: bool,
    pub condition: ModuleCondition,
    pub manifest: Option<Arc<ModuleManifest>>,
    pub manifest_hash: Option<[u8; 32]>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl ModuleRuntime {
    pub fn unavailable(spec: ModuleSpec, enabled: bool) -> Self {
        Self {
            spec,
            enabled,
            condition: ModuleCondition::Unavailable,
            manifest: None,
            manifest_hash: None,
            last_seen_at: None,
            error: Some("service has not published a valid Manifest".to_string()),
        }
    }

    pub fn compatible(&self) -> bool {
        self.manifest.is_some() && self.condition != ModuleCondition::Incompatible
    }

    pub fn available(&self) -> bool {
        self.enabled && self.condition == ModuleCondition::Healthy
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleStatusResponse {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub available: bool,
    pub compatible: bool,
    pub release_version: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl From<&ModuleRuntime> for ModuleStatusResponse {
    fn from(runtime: &ModuleRuntime) -> Self {
        Self {
            id: runtime.spec.id.to_string(),
            name: runtime.spec.name.to_string(),
            enabled: runtime.enabled,
            available: runtime.available(),
            compatible: runtime.compatible(),
            release_version: runtime
                .manifest
                .as_deref()
                .map(|manifest| manifest.release_version.clone()),
            last_seen_at: runtime.last_seen_at,
            error: runtime.error.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ModuleHealthResponse {
    pub module: &'static str,
    pub available: bool,
    pub release_version: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateModuleRequest {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMenuResponse {
    pub module: String,
    pub module_name: String,
    pub code: String,
    pub title: String,
    pub path: String,
    pub icon: String,
    pub sort_order: i32,
    pub permission: String,
}

#[derive(Debug, Clone)]
pub struct GatewayTarget {
    pub module: String,
    pub base_url: String,
    pub access: AccessMode,
    pub permission: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GatewayLookup {
    Found,
    NotFound,
    MethodNotAllowed,
    ServiceUnavailable,
}

#[cfg(test)]
mod tests {
    use super::ModuleHealthResponse;

    #[test]
    fn dashboard_health_keeps_its_wire_contract() {
        assert_eq!(
            serde_json::to_value(ModuleHealthResponse {
                module: "monitor",
                available: true,
                release_version: Some("1.2.3".to_string()),
            })
            .expect("serialize module health"),
            serde_json::json!({
                "module": "monitor",
                "available": true,
                "releaseVersion": "1.2.3",
            })
        );
    }
}
