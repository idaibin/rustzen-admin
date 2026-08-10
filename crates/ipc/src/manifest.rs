use std::collections::{BTreeMap, BTreeSet};

use http::Method;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::delegation::{DelegationError, normalize_path, validate_capability, validate_module_id};

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct ModuleDefinition {
    pub module: ModuleMetadata,
    #[serde(default)]
    pub menus: Vec<MenuDefinition>,
}

impl ModuleDefinition {
    pub fn from_toml(source: &str) -> Result<Self, ManifestError> {
        let definition: Self = toml::from_str(source)?;
        definition.validate_identity()?;
        Ok(definition)
    }

    pub fn build_manifest(
        &self,
        release_version: impl Into<String>,
        mut routes: Vec<RouteManifest>,
    ) -> Result<ModuleManifest, ManifestError> {
        self.validate_identity()?;
        validate_catalog(&self.module.id, &self.menus, &routes)?;

        routes.sort_by(|left, right| {
            left.method.cmp(&right.method).then_with(|| left.path.cmp(&right.path))
        });
        let mut menus = self.menus.clone();
        menus.sort_by(|left, right| {
            left.sort_order.cmp(&right.sort_order).then_with(|| left.code.cmp(&right.code))
        });
        Ok(ModuleManifest {
            module: self.module.id.clone(),
            name: self.module.name.clone(),
            api_prefix: self.module.api_prefix.clone(),
            contract_version: self.module.contract_version,
            release_version: release_version.into(),
            menus,
            routes,
        })
    }

    fn validate_identity(&self) -> Result<(), ManifestError> {
        validate_module_id(&self.module.id)?;
        if self.module.name.trim().is_empty() {
            return Err(ManifestError::InvalidName);
        }
        if self.module.api_prefix != format!("/api/{}", self.module.id) {
            return Err(ManifestError::InvalidApiPrefix);
        }
        if self.module.contract_version != crate::CONTRACT_VERSION {
            return Err(ManifestError::UnsupportedContractVersion(self.module.contract_version));
        }
        let mut menu_codes = BTreeSet::new();
        for menu in &self.menus {
            if menu.code.trim().is_empty()
                || menu.title.trim().is_empty()
                || !menu.path.starts_with('/')
                || !menu_codes.insert(menu.code.as_str())
            {
                return Err(ManifestError::InvalidMenu(menu.code.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Eq, PartialEq)]
pub struct ModuleMetadata {
    pub id: String,
    pub name: String,
    pub api_prefix: String,
    pub contract_version: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MenuDefinition {
    pub code: String,
    pub title: String,
    pub path: String,
    pub icon: String,
    #[serde(alias = "sort_order")]
    pub sort_order: i32,
    pub permission: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AccessMode {
    Protected,
    Public,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RouteManifest {
    pub method: String,
    pub path: String,
    pub access: AccessMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
}

impl RouteManifest {
    pub fn protected(
        method: Method,
        path: impl Into<String>,
        permission: impl Into<String>,
    ) -> Self {
        Self {
            method: method.as_str().to_string(),
            path: path.into(),
            access: AccessMode::Protected,
            permission: Some(permission.into()),
        }
    }

    pub fn public(method: Method, path: impl Into<String>) -> Self {
        Self {
            method: method.as_str().to_string(),
            path: path.into(),
            access: AccessMode::Public,
            permission: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModuleManifest {
    pub module: String,
    pub name: String,
    pub api_prefix: String,
    pub contract_version: u32,
    pub release_version: String,
    pub menus: Vec<MenuDefinition>,
    pub routes: Vec<RouteManifest>,
}

impl ModuleManifest {
    pub fn validate(&self) -> Result<(), ManifestError> {
        let definition = ModuleDefinition {
            module: ModuleMetadata {
                id: self.module.clone(),
                name: self.name.clone(),
                api_prefix: self.api_prefix.clone(),
                contract_version: self.contract_version,
            },
            menus: self.menus.clone(),
        };
        definition.validate_identity()?;
        if self.release_version.trim().is_empty() {
            return Err(ManifestError::InvalidReleaseVersion);
        }
        validate_catalog(&self.module, &self.menus, &self.routes)
    }

    pub fn to_deterministic_json(&self) -> Result<Vec<u8>, ManifestError> {
        self.validate()?;
        Ok(serde_json::to_vec(self)?)
    }
}

fn validate_catalog(
    module: &str,
    menus: &[MenuDefinition],
    routes: &[RouteManifest],
) -> Result<(), ManifestError> {
    validate_routes(module, routes)?;
    let capabilities =
        routes.iter().filter_map(|route| route.permission.as_deref()).collect::<BTreeSet<_>>();
    for menu in menus {
        validate_capability(module, &menu.permission)?;
        if !capabilities.contains(menu.permission.as_str()) {
            return Err(ManifestError::UnknownMenuCapability(menu.permission.clone()));
        }
    }
    Ok(())
}

pub(crate) fn validate_routes(module: &str, routes: &[RouteManifest]) -> Result<(), ManifestError> {
    let mut patterns_by_method: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for route in routes {
        validate_method(&route.method)?;
        validate_route_pattern(&route.path)?;
        match (&route.access, &route.permission) {
            (AccessMode::Protected, Some(permission)) => {
                validate_capability(module, permission)?;
            }
            (AccessMode::Public, None) => {}
            _ => return Err(ManifestError::InvalidRouteAccess(route.path.clone())),
        }
        let patterns = patterns_by_method.entry(route.method.as_str()).or_default();
        if patterns.iter().any(|existing| patterns_overlap(existing, &route.path)) {
            return Err(ManifestError::AmbiguousRoute {
                method: route.method.clone(),
                path: route.path.clone(),
            });
        }
        patterns.push(&route.path);
    }
    Ok(())
}

fn validate_method(method: &str) -> Result<(), ManifestError> {
    if matches!(method, "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "OPTIONS") {
        Ok(())
    } else {
        Err(ManifestError::UnsupportedMethod(method.to_string()))
    }
}

fn validate_route_pattern(path: &str) -> Result<(), ManifestError> {
    normalize_path(path)?;
    if path == "/" || path.ends_with('/') {
        return Err(ManifestError::InvalidRoutePattern(path.to_string()));
    }
    let mut parameters = BTreeSet::new();
    for segment in path.trim_start_matches('/').split('/') {
        if segment.is_empty() {
            return Err(ManifestError::InvalidRoutePattern(path.to_string()));
        }
        if segment.starts_with('{') || segment.ends_with('}') {
            let Some(parameter) =
                segment.strip_prefix('{').and_then(|value| value.strip_suffix('}'))
            else {
                return Err(ManifestError::InvalidRoutePattern(path.to_string()));
            };
            if parameter.is_empty()
                || !parameter.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                || !parameters.insert(parameter)
            {
                return Err(ManifestError::InvalidRoutePattern(path.to_string()));
            }
        } else if segment.contains(['{', '}', '*', ':']) {
            return Err(ManifestError::InvalidRoutePattern(path.to_string()));
        }
    }
    Ok(())
}

fn patterns_overlap(left: &str, right: &str) -> bool {
    let left = left.trim_start_matches('/').split('/').collect::<Vec<_>>();
    let right = right.trim_start_matches('/').split('/').collect::<Vec<_>>();
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left == &right || is_parameter(left) || is_parameter(right))
}

fn is_parameter(segment: &str) -> bool {
    segment.starts_with('{') && segment.ends_with('}')
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error(transparent)]
    Delegation(#[from] DelegationError),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("module name is invalid")]
    InvalidName,
    #[error("module API prefix must match /api/<module>")]
    InvalidApiPrefix,
    #[error("unsupported contract version {0}")]
    UnsupportedContractVersion(u32),
    #[error("release version is invalid")]
    InvalidReleaseVersion,
    #[error("module menu {0:?} is invalid or duplicated")]
    InvalidMenu(String),
    #[error("menu permission {0:?} is not declared by a protected route")]
    UnknownMenuCapability(String),
    #[error("HTTP method {0} is not supported")]
    UnsupportedMethod(String),
    #[error("route pattern {0:?} is invalid")]
    InvalidRoutePattern(String),
    #[error("route access declaration for {0:?} is invalid")]
    InvalidRouteAccess(String),
    #[error("route {method} {path} duplicates or overlaps another route")]
    AmbiguousRoute { method: String, path: String },
}

#[cfg(test)]
mod tests {
    use http::Method;

    use super::{ManifestError, ModuleDefinition, RouteManifest};

    const MODULE_TOML: &str = r#"
[module]
id = "reports"
name = "Reports"
api_prefix = "/api/reports"
contract_version = 1

[[menus]]
code = "reports"
title = "Reports"
path = "/reports"
icon = "file-text"
sort_order = 30
permission = "reports:view"
"#;

    #[test]
    fn manifest_generation_is_deterministic_and_keeps_public_routes_explicit() {
        let definition = ModuleDefinition::from_toml(MODULE_TOML).expect("definition");
        let first = definition
            .build_manifest(
                "0.5.0",
                vec![
                    RouteManifest::public(Method::POST, "/track"),
                    RouteManifest::protected(Method::POST, "/jobs", "reports:manage"),
                    RouteManifest::protected(Method::GET, "/jobs", "reports:view"),
                ],
            )
            .expect("manifest");
        let second = definition
            .build_manifest(
                "0.5.0",
                vec![
                    RouteManifest::protected(Method::GET, "/jobs", "reports:view"),
                    RouteManifest::public(Method::POST, "/track"),
                    RouteManifest::protected(Method::POST, "/jobs", "reports:manage"),
                ],
            )
            .expect("manifest");

        assert_eq!(first, second);
        assert_eq!(
            first.to_deterministic_json().expect("json"),
            second.to_deterministic_json().expect("json")
        );
        assert!(first.routes.iter().any(|route| route.permission.is_none()));
    }

    #[test]
    fn duplicate_and_ambiguous_routes_are_rejected() {
        let definition = ModuleDefinition::from_toml(MODULE_TOML).expect("definition");
        let result = definition.build_manifest(
            "0.5.0",
            vec![
                RouteManifest::protected(Method::GET, "/jobs/{job_id}", "reports:view"),
                RouteManifest::protected(Method::GET, "/jobs/{id}", "reports:view"),
            ],
        );
        assert!(matches!(result, Err(ManifestError::AmbiguousRoute { .. })));

        let result = definition.build_manifest(
            "0.5.0",
            vec![
                RouteManifest::public(Method::OPTIONS, "/track"),
                RouteManifest::public(Method::OPTIONS, "/track"),
            ],
        );
        assert!(matches!(result, Err(ManifestError::AmbiguousRoute { .. })));
    }

    #[test]
    fn options_is_the_only_additional_public_method() {
        let definition = ModuleDefinition::from_toml(MODULE_TOML).expect("definition");
        assert!(
            definition
                .build_manifest(
                    "0.5.0",
                    vec![
                        RouteManifest::public(Method::OPTIONS, "/track"),
                        RouteManifest::protected(Method::GET, "/jobs", "reports:view"),
                    ],
                )
                .is_ok()
        );
        let result = definition
            .build_manifest("0.5.0", vec![RouteManifest::public(Method::TRACE, "/track")]);
        assert!(
            matches!(result, Err(ManifestError::UnsupportedMethod(method)) if method == "TRACE")
        );
    }

    #[test]
    fn cross_module_and_unknown_menu_capabilities_are_rejected() {
        let definition = ModuleDefinition::from_toml(MODULE_TOML).expect("definition");
        assert!(
            definition
                .build_manifest(
                    "0.5.0",
                    vec![RouteManifest::protected(Method::GET, "/jobs", "insights:view")],
                )
                .is_err()
        );
        assert!(
            definition
                .build_manifest(
                    "0.5.0",
                    vec![RouteManifest::protected(Method::GET, "/jobs", "reports:manage")],
                )
                .is_err()
        );
    }

    #[test]
    fn received_manifest_reuses_generation_validation() {
        let definition = ModuleDefinition::from_toml(MODULE_TOML).expect("definition");
        let mut manifest = definition
            .build_manifest(
                "0.5.0",
                vec![RouteManifest::protected(Method::GET, "/jobs", "reports:view")],
            )
            .expect("manifest");
        assert!(manifest.validate().is_ok());

        manifest.routes[0].permission = Some("insights:view".to_string());
        assert!(manifest.validate().is_err());
    }
}
