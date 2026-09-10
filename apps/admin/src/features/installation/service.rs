use rustzen_auth::auth::CurrentUser;

use crate::features::modules::{
    service::ModuleControlState,
    types::{ModuleCondition, selected_module_id},
};

use super::types::{InstallationResponse, SelectedServiceResponse, SelectedServiceState};

const PACKAGED_WEB_DIGEST: &str = env!("RUSTZEN_PACKAGED_WEB_DIGEST");
const PACKAGED_WEB_COMPOSITION_ID: &str = env!("RUSTZEN_PACKAGED_WEB_COMPOSITION_ID");

#[derive(Clone)]
pub struct InstallationState {
    modules: ModuleControlState,
    build_id: String,
    composition_id: String,
}

impl InstallationState {
    pub fn load(modules: ModuleControlState) -> Result<Self, String> {
        Self::new(
            modules,
            required_sha256_env("RUSTZEN_BUILD_ID")?,
            required_sha256_env("RUSTZEN_COMPOSITION_ID")?,
        )
    }

    fn new(
        modules: ModuleControlState,
        build_id: String,
        composition_id: String,
    ) -> Result<Self, String> {
        if !is_sha256(&build_id) || !is_sha256(&composition_id) {
            return Err("Admin installation identity contains an invalid digest".into());
        }
        if composition_id != PACKAGED_WEB_COMPOSITION_ID {
            return Err("packaged Web composition differs from installed Admin identity".into());
        }
        Ok(Self { modules, build_id, composition_id })
    }

    pub fn response(&self, user: &CurrentUser) -> InstallationResponse {
        let snapshot = self.modules.registry.snapshot();
        let mut services = snapshot
            .modules()
            .values()
            .map(|runtime| SelectedServiceResponse {
                id: runtime.spec.id.to_owned(),
                state: if runtime.available() {
                    SelectedServiceState::Healthy
                } else if runtime.condition == ModuleCondition::Incompatible {
                    SelectedServiceState::Incompatible
                } else {
                    SelectedServiceState::Unavailable
                },
                release_version: runtime
                    .manifest
                    .as_deref()
                    .map(|manifest| manifest.release_version.clone()),
            })
            .collect::<Vec<_>>();
        services.sort_by(|left, right| left.id.cmp(&right.id));
        let mut capabilities = user.permissions.iter().cloned().collect::<Vec<_>>();
        capabilities.sort();
        InstallationResponse {
            release_version: env!("CARGO_PKG_VERSION"),
            composition_id: self.composition_id.clone(),
            build_id: self.build_id.clone(),
            web_digest: PACKAGED_WEB_DIGEST,
            feature_ids: selected_feature_ids(),
            services,
            capabilities,
        }
    }

    #[cfg(test)]
    pub fn for_test(modules: ModuleControlState) -> Self {
        Self::new(modules, "1".repeat(64), PACKAGED_WEB_COMPOSITION_ID.to_owned())
            .expect("test installation identity")
    }
}

pub fn web_digest() -> &'static str {
    PACKAGED_WEB_DIGEST
}

fn selected_feature_ids() -> Vec<&'static str> {
    let features = vec!["access", selected_module_id()];
    #[cfg(feature = "notifications")]
    let features = {
        let mut features = features;
        features.push("notifications");
        features
    };
    features
}

fn required_sha256_env(name: &str) -> Result<String, String> {
    std::env::var(name)
        .ok()
        .filter(|value| is_sha256(value))
        .ok_or_else(|| format!("{name} is invalid"))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod identity_tests {
    use super::is_sha256;

    #[test]
    fn installation_digests_require_lowercase_sha256() {
        assert!(is_sha256(&"a".repeat(64)));
        assert!(!is_sha256(&"A".repeat(64)));
        assert!(!is_sha256(&"g".repeat(64)));
        assert!(!is_sha256(&"a".repeat(63)));
    }
}
