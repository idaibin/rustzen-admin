use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
};

use axum::http::Method;
use rustzen_ipc::{AccessMode, ModuleManifest};

#[cfg(feature = "full")]
use super::types::ModuleStatusResponse;
use super::types::{GatewayLookup, GatewayTarget, ModuleRuntime, ModuleSpec};

#[derive(Clone)]
pub struct ModuleRegistry {
    current: Arc<RwLock<Arc<RegistrySnapshot>>>,
}

impl ModuleRegistry {
    pub fn new(specs: Vec<ModuleSpec>, enabled: &BTreeMap<String, bool>) -> Self {
        let modules = specs
            .into_iter()
            .map(|spec| {
                let is_enabled = enabled.get(spec.id).copied().unwrap_or(true);
                (spec.id.to_string(), ModuleRuntime::unavailable(spec, is_enabled))
            })
            .collect();
        Self { current: Arc::new(RwLock::new(Arc::new(RegistrySnapshot::from_modules(modules)))) }
    }

    pub fn snapshot(&self) -> Arc<RegistrySnapshot> {
        self.current
            .read()
            .map(|snapshot| Arc::clone(&snapshot))
            .unwrap_or_else(|poisoned| Arc::clone(&poisoned.into_inner()))
    }

    #[cfg(all(test, feature = "full"))]
    pub fn replace(&self, snapshot: RegistrySnapshot) {
        let replacement = Arc::new(snapshot);
        match self.current.write() {
            Ok(mut current) => *current = replacement,
            Err(poisoned) => *poisoned.into_inner() = replacement,
        }
    }

    pub fn update_module(&self, module: &str, update: impl FnOnce(&mut ModuleRuntime)) -> bool {
        let mut current = match self.current.write() {
            Ok(current) => current,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut modules = current.modules.clone();
        let Some(runtime) = modules.get_mut(module) else {
            return false;
        };
        update(runtime);
        *current = Arc::new(RegistrySnapshot::from_modules(modules));
        true
    }
}

#[derive(Debug, Clone)]
pub struct RegistrySnapshot {
    modules: BTreeMap<String, ModuleRuntime>,
    routes: Vec<CompiledRoute>,
}

impl RegistrySnapshot {
    pub fn from_modules(modules: BTreeMap<String, ModuleRuntime>) -> Self {
        let routes = modules
            .values()
            .filter_map(|runtime| runtime.manifest.as_deref().map(|manifest| (runtime, manifest)))
            .flat_map(|(runtime, manifest)| compile_routes(runtime, manifest))
            .collect();
        Self { modules, routes }
    }

    pub fn modules(&self) -> &BTreeMap<String, ModuleRuntime> {
        &self.modules
    }

    /// Returns registered modules in the product's fixed display order.
    ///
    /// `BTreeMap` keeps registry lookups deterministic, but its lexical key ordering is not the
    /// order that the module APIs expose. Unknown modules are retained after the fixed modules so
    /// test fixtures and future extension modules remain observable.
    #[cfg(feature = "full")]
    pub fn display_modules(&self) -> Vec<&ModuleRuntime> {
        let specs = ModuleSpec::fixed();
        let mut modules =
            specs.iter().filter_map(|spec| self.modules.get(spec.id)).collect::<Vec<_>>();
        modules.extend(
            self.modules
                .iter()
                .filter(|(id, _)| !specs.iter().any(|spec| spec.id == id.as_str()))
                .map(|(_, runtime)| runtime),
        );
        modules
    }

    #[cfg(all(test, feature = "full"))]
    pub fn into_modules(self) -> BTreeMap<String, ModuleRuntime> {
        self.modules
    }

    #[cfg(feature = "full")]
    pub fn statuses(&self) -> Vec<ModuleStatusResponse> {
        self.display_modules().into_iter().map(ModuleStatusResponse::from).collect()
    }

    pub fn lookup(&self, method: &Method, path: &str) -> (GatewayLookup, Option<GatewayTarget>) {
        if !is_canonical_request_path(path) {
            return (GatewayLookup::NotFound, None);
        }
        let Some(module_id) = module_from_path(path) else {
            return (GatewayLookup::NotFound, None);
        };
        let Some(runtime) = self.modules.get(module_id) else {
            return (GatewayLookup::NotFound, None);
        };
        if runtime.manifest.is_none() {
            return (GatewayLookup::ServiceUnavailable, None);
        }

        let path_routes = self.routes.iter().filter(|route| route.matches_path(path));
        let mut method_mismatch = false;
        for route in path_routes {
            if route.method != *method {
                method_mismatch = true;
                continue;
            }
            if !runtime.enabled || !runtime.available() {
                return (GatewayLookup::ServiceUnavailable, None);
            }
            return (
                GatewayLookup::Found,
                Some(GatewayTarget {
                    module: route.module.clone(),
                    base_url: route.base_url.clone(),
                    access: route.access,
                    permission: route.permission.clone(),
                }),
            );
        }
        if method_mismatch {
            (GatewayLookup::MethodNotAllowed, None)
        } else {
            (GatewayLookup::NotFound, None)
        }
    }
}

#[derive(Debug, Clone)]
struct CompiledRoute {
    module: String,
    base_url: String,
    method: Method,
    segments: Vec<RouteSegment>,
    access: AccessMode,
    permission: Option<String>,
}

impl CompiledRoute {
    fn matches_path(&self, path: &str) -> bool {
        let segments = split_path(path).collect::<Vec<_>>();
        self.segments.len() == segments.len()
            && self.segments.iter().zip(&segments).all(|(expected, actual)| match expected {
                RouteSegment::Static(expected) => expected == actual,
                RouteSegment::Parameter => !actual.is_empty(),
            })
    }
}

#[derive(Debug, Clone)]
enum RouteSegment {
    Static(String),
    Parameter,
}

fn compile_routes<'a>(
    runtime: &'a ModuleRuntime,
    manifest: &'a ModuleManifest,
) -> impl Iterator<Item = CompiledRoute> + 'a {
    manifest.routes.iter().map(|route| {
        let full_path = format!("{}{}", manifest.api_prefix, route.path);
        CompiledRoute {
            module: runtime.spec.id.to_string(),
            base_url: runtime.spec.base_url.clone(),
            method: Method::from_bytes(route.method.as_bytes()).expect("validated Manifest method"),
            segments: split_path(&full_path)
                .map(|segment| {
                    if segment.starts_with('{') && segment.ends_with('}') {
                        RouteSegment::Parameter
                    } else {
                        RouteSegment::Static(segment.to_string())
                    }
                })
                .collect(),
            access: route.access,
            permission: route.permission.clone(),
        }
    })
}

fn split_path(path: &str) -> impl Iterator<Item = &str> {
    path.trim_start_matches('/').split('/').filter(|segment| !segment.is_empty())
}

fn is_canonical_request_path(path: &str) -> bool {
    path.starts_with('/')
        && path.len() > 1
        && !path.ends_with('/')
        && !path.contains("//")
        && !path.split('/').any(|segment| matches!(segment, "." | ".."))
}

fn module_from_path(path: &str) -> Option<&str> {
    let mut segments = split_path(path);
    (segments.next() == Some("api")).then(|| segments.next()).flatten()
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use axum::http::Method;
    use rustzen_ipc::{AccessMode, ModuleManifest, RouteManifest};

    use super::{GatewayLookup, ModuleRegistry, RegistrySnapshot};
    use crate::features::modules::types::{ModuleCondition, ModuleRuntime, ModuleSpec};

    fn healthy_runtime() -> ModuleRuntime {
        ModuleRuntime {
            spec: ModuleSpec {
                id: "reports",
                name: "Reports",
                base_url: "http://127.0.0.1:9804".to_string(),
            },
            enabled: true,
            condition: ModuleCondition::Healthy,
            manifest: Some(Arc::new(ModuleManifest {
                module: "reports".to_string(),
                name: "Reports".to_string(),
                api_prefix: "/api/reports".to_string(),
                contract_version: 1,
                release_version: env!("CARGO_PKG_VERSION").to_string(),
                menus: Vec::new(),
                routes: vec![RouteManifest {
                    method: "GET".to_string(),
                    path: "/jobs/{job_id}".to_string(),
                    access: AccessMode::Protected,
                    permission: Some("reports:view".to_string()),
                }],
            })),
            manifest_hash: Some([1; 32]),
            storage: None,
            last_seen_at: Some(chrono::Utc::now()),
            error: None,
        }
    }

    #[test]
    fn method_and_parameter_matching_are_in_memory_and_exact() {
        let snapshot = RegistrySnapshot::from_modules(BTreeMap::from([(
            "reports".to_string(),
            healthy_runtime(),
        )]));
        let (lookup, target) = snapshot.lookup(&Method::GET, "/api/reports/jobs/42");
        assert_eq!(lookup, GatewayLookup::Found);
        assert_eq!(target.expect("target").permission.as_deref(), Some("reports:view"));
        assert_eq!(
            snapshot.lookup(&Method::POST, "/api/reports/jobs/42").0,
            GatewayLookup::MethodNotAllowed
        );
        assert_eq!(
            snapshot.lookup(&Method::GET, "/api/reports/jobs/42/extra").0,
            GatewayLookup::NotFound
        );
        for path in ["/api/reports//jobs/42", "/api/reports/jobs/42/", "/api/reports/../jobs/42"] {
            assert_eq!(snapshot.lookup(&Method::GET, path).0, GatewayLookup::NotFound, "{path}");
        }
    }

    #[test]
    fn disabled_and_unavailable_modules_return_service_unavailable() {
        let mut runtime = healthy_runtime();
        runtime.enabled = false;
        let disabled =
            RegistrySnapshot::from_modules(BTreeMap::from([("reports".to_string(), runtime)]));
        assert_eq!(
            disabled.lookup(&Method::GET, "/api/reports/jobs/42").0,
            GatewayLookup::ServiceUnavailable
        );

        let registry = ModuleRegistry::new(
            vec![ModuleSpec {
                id: "reports",
                name: "Reports",
                base_url: "http://127.0.0.1:9804".to_string(),
            }],
            &BTreeMap::new(),
        );
        assert_eq!(
            registry.snapshot().lookup(&Method::GET, "/api/reports/jobs/42").0,
            GatewayLookup::ServiceUnavailable
        );
    }
}
