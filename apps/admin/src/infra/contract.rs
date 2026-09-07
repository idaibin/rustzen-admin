//! Bounded, code-first route contract registration for Admin routes.
//!
//! This is deliberately local to Admin: it turns one typed route registration into
//! both the Axum route and the metadata later consumed by the OpenAPI exporter.

use axum::{Router, http::Method, routing::MethodRouter};
use rustzen_auth::permission::{PermissionsCheck, RouterExt};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(
    dead_code,
    reason = "Public, Any, and All are validated by the contract harness before a production route needs them"
)]
pub enum AccessPolicy {
    Public,
    Authenticated,
    Require(&'static str),
    Any(Vec<&'static str>),
    All(Vec<&'static str>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RouteContract {
    pub method: Method,
    pub path: String,
    pub operation: OperationDescriptor,
    pub access: RegisteredAccess,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code, reason = "synthetic policy contracts are exercised by focused tests")]
pub enum OperationDescriptor {
    Login,
    CurrentAdminUser,
    Logout,
    UpdateAccountAvatar,
    UpdateAccountProfile,
    ChangeAccountPassword,
    GetDashboardStats,
    ListManageLogs,
    ExportManageLogs,
    ListManageTasks,
    ListTaskRuns,
    RunTask,
    ListDeployments,
    UploadDeployment,
    CleanupDeployments,
    GetDeployment,
    ExpireDeployment,
    DeleteDeployment,
    DeployVersion,
    ListMenus,
    ListModuleMenuInventory,
    UpdateMenu,
    DeleteMenu,
    GetMenuOptions,
    ListRoles,
    CreateRole,
    UpdateRole,
    DeleteRole,
    GetRoleOptions,
    GetStatusOverview,
    ListModuleLogs,
    TailModuleLog,
    BackupModuleLogs,
    PreviewModuleLogCleanup,
    ConfirmModuleLogCleanup,
    ListUsers,
    CreateAdminUser,
    UpdateUser,
    DeleteUser,
    GetUserOptions,
    GetUserStatusOptions,
    UpdateUserPassword,
    UpdateUserStatus,
    ListModules,
    UpdateModuleEnabled,
    GetModuleNavigation,
    GetDashboardModules,
    ListNotifications,
    GetNotificationUnreadCount,
    GetNotification,
    ReadNotification,
    ReadAllNotifications,
    ContractPublic,
    ContractAny,
    ContractAll,
    BenchmarkRequire,
}
impl OperationDescriptor {
    pub const fn operation_id(&self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::CurrentAdminUser => "getCurrentAdminUser",
            Self::Logout => "logout",
            Self::UpdateAccountAvatar => "updateAccountAvatar",
            Self::UpdateAccountProfile => "updateAccountProfile",
            Self::ChangeAccountPassword => "changeAccountPassword",
            Self::GetDashboardStats => "getDashboardStats",
            Self::ListManageLogs => "listManageLogs",
            Self::ExportManageLogs => "exportManageLogs",
            Self::ListManageTasks => "listManageTasks",
            Self::ListTaskRuns => "listTaskRuns",
            Self::RunTask => "runTask",
            Self::ListDeployments => "listDeployments",
            Self::UploadDeployment => "uploadDeployment",
            Self::CleanupDeployments => "cleanupDeployments",
            Self::GetDeployment => "getDeployment",
            Self::ExpireDeployment => "expireDeployment",
            Self::DeleteDeployment => "deleteDeployment",
            Self::DeployVersion => "deployVersion",
            Self::ListMenus => "listMenus",
            Self::ListModuleMenuInventory => "listModuleMenuInventory",
            Self::UpdateMenu => "updateMenu",
            Self::DeleteMenu => "deleteMenu",
            Self::GetMenuOptions => "getMenuOptions",
            Self::ListRoles => "listRoles",
            Self::CreateRole => "createRole",
            Self::UpdateRole => "updateRole",
            Self::DeleteRole => "deleteRole",
            Self::GetRoleOptions => "getRoleOptions",
            Self::GetStatusOverview => "getStatusOverview",
            Self::ListModuleLogs => "listModuleLogs",
            Self::TailModuleLog => "tailModuleLog",
            Self::BackupModuleLogs => "backupModuleLogs",
            Self::PreviewModuleLogCleanup => "previewModuleLogCleanup",
            Self::ConfirmModuleLogCleanup => "confirmModuleLogCleanup",
            Self::ListUsers => "listUsers",
            Self::CreateAdminUser => "createAdminUser",
            Self::UpdateUser => "updateUser",
            Self::DeleteUser => "deleteUser",
            Self::GetUserOptions => "getUserOptions",
            Self::GetUserStatusOptions => "getUserStatusOptions",
            Self::UpdateUserPassword => "updateUserPassword",
            Self::UpdateUserStatus => "updateUserStatus",
            Self::ListModules => "listModules",
            Self::UpdateModuleEnabled => "updateModuleEnabled",
            Self::GetModuleNavigation => "getModuleNavigation",
            Self::GetDashboardModules => "getDashboardModules",
            Self::ListNotifications => "listNotifications",
            Self::GetNotificationUnreadCount => "getNotificationUnreadCount",
            Self::GetNotification => "getNotification",
            Self::ReadNotification => "readNotification",
            Self::ReadAllNotifications => "readAllNotifications",
            Self::ContractPublic => "contractPublic",
            Self::ContractAny => "contractAny",
            Self::ContractAll => "contractAll",
            Self::BenchmarkRequire => "benchRequire",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RegisteredAccess {
    Public,
    Authenticated,
    Require(Vec<String>),
    Any(Vec<String>),
    All(Vec<String>),
}

#[derive(Debug, thiserror::Error, Eq, PartialEq)]
pub enum ContractError {
    #[error("capability collection must not be empty")]
    EmptyCapabilities,
    #[error("capability must not be empty")]
    EmptyCapability,
    #[error("route already registered for {method} {path}")]
    RouteConflict { method: Method, path: String },
    #[error("operation id already registered: {0}")]
    OperationIdConflict(String),
}

/// The route and its contract are registered together. The outer JWT layer is
/// intentionally still assembled in `infra::app`; this type records that fact
/// but does not duplicate it.
pub struct ContractRouter<S> {
    router: Router<S>,
    contracts: Vec<RouteContract>,
    prefix: String,
}

impl<S> ContractRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self { router: Router::new(), contracts: Vec::new(), prefix: "/".to_owned() }
    }

    pub fn nest(mut self, path: &str, child: Self) -> Result<Self, ContractError> {
        let path = normalize_path(path);
        let mut contracts = child.contracts;
        for contract in &mut contracts {
            contract.path = join_path(&path, &contract.path);
        }
        for contract in &contracts {
            if self
                .contracts
                .iter()
                .any(|item| item.method == contract.method && item.path == contract.path)
            {
                return Err(ContractError::RouteConflict {
                    method: contract.method.clone(),
                    path: contract.path.clone(),
                });
            }
            if self
                .contracts
                .iter()
                .any(|item| item.operation.operation_id() == contract.operation.operation_id())
            {
                return Err(ContractError::OperationIdConflict(
                    contract.operation.operation_id().to_owned(),
                ));
            }
        }
        self.router = self.router.nest(&path, child.router);
        self.contracts.extend(contracts);
        Ok(self)
    }

    pub fn get(
        self,
        path: &str,
        operation: OperationDescriptor,
        access: AccessPolicy,
        route: MethodRouter<S>,
    ) -> Result<Self, ContractError> {
        self.register(Method::GET, path, operation, access, route)
    }

    pub fn post(
        self,
        path: &str,
        operation: OperationDescriptor,
        access: AccessPolicy,
        route: MethodRouter<S>,
    ) -> Result<Self, ContractError> {
        self.register(Method::POST, path, operation, access, route)
    }

    pub fn put(
        self,
        path: &str,
        operation: OperationDescriptor,
        access: AccessPolicy,
        route: MethodRouter<S>,
    ) -> Result<Self, ContractError> {
        self.register(Method::PUT, path, operation, access, route)
    }

    #[allow(dead_code, reason = "the contract surface supports PATCH registrations")]
    pub fn patch(
        self,
        path: &str,
        operation: OperationDescriptor,
        access: AccessPolicy,
        route: MethodRouter<S>,
    ) -> Result<Self, ContractError> {
        self.register(Method::PATCH, path, operation, access, route)
    }

    pub fn delete(
        self,
        path: &str,
        operation: OperationDescriptor,
        access: AccessPolicy,
        route: MethodRouter<S>,
    ) -> Result<Self, ContractError> {
        self.register(Method::DELETE, path, operation, access, route)
    }

    fn register(
        mut self,
        method: Method,
        path: &str,
        operation: OperationDescriptor,
        access: AccessPolicy,
        route: MethodRouter<S>,
    ) -> Result<Self, ContractError> {
        let route_path = normalize_path(path);
        let path = join_path(&self.prefix, &route_path);
        let (access, permission_check) = normalize_access(access)?;
        if self.contracts.iter().any(|item| item.method == method && item.path == path) {
            return Err(ContractError::RouteConflict { method, path });
        }
        if self
            .contracts
            .iter()
            .any(|item| item.operation.operation_id() == operation.operation_id())
        {
            return Err(ContractError::OperationIdConflict(operation.operation_id().to_owned()));
        }

        self.router = match permission_check {
            Some(permission_check) => {
                self.router.route_with_permission(&route_path, route, permission_check)
            }
            None => self.router.route(&route_path, route),
        };
        self.contracts.push(RouteContract { method, path, operation, access });
        Ok(self)
    }

    pub fn into_parts(self) -> (Router<S>, Vec<RouteContract>) {
        (self.router, self.contracts)
    }
}

fn normalize_access(
    access: AccessPolicy,
) -> Result<(RegisteredAccess, Option<PermissionsCheck>), ContractError> {
    fn normalize(codes: Vec<&'static str>) -> Result<Vec<&'static str>, ContractError> {
        if codes.is_empty() {
            return Err(ContractError::EmptyCapabilities);
        }
        let mut normalized = BTreeSet::new();
        for code in codes {
            let code = code.trim();
            if code.is_empty() {
                return Err(ContractError::EmptyCapability);
            }
            normalized.insert(code);
        }
        Ok(normalized.into_iter().collect())
    }

    match access {
        AccessPolicy::Public => Ok((RegisteredAccess::Public, None)),
        AccessPolicy::Authenticated => Ok((RegisteredAccess::Authenticated, None)),
        AccessPolicy::Require(code) => {
            let codes = normalize(vec![code])?;
            Ok((
                RegisteredAccess::Require(codes.iter().map(|code| (*code).to_owned()).collect()),
                Some(PermissionsCheck::Require(codes[0])),
            ))
        }
        AccessPolicy::Any(codes) => {
            let codes = normalize(codes)?;
            Ok((
                RegisteredAccess::Any(codes.iter().map(|code| (*code).to_owned()).collect()),
                Some(PermissionsCheck::Any(codes)),
            ))
        }
        AccessPolicy::All(codes) => {
            let codes = normalize(codes)?;
            Ok((
                RegisteredAccess::All(codes.iter().map(|code| (*code).to_owned()).collect()),
                Some(PermissionsCheck::All(codes)),
            ))
        }
    }
}

pub fn join_path(prefix: &str, path: &str) -> String {
    normalize_path(&format!("{}/{}", prefix.trim_matches('/'), path.trim_matches('/')))
}

fn normalize_path(path: &str) -> String {
    let segments = path.split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
    if segments.is_empty() { "/".to_owned() } else { format!("/{}", segments.join("/")) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    async fn ok() -> StatusCode {
        StatusCode::NO_CONTENT
    }

    #[tokio::test]
    async fn red_then_green_public_route_is_registered_with_a_normalized_contract() {
        let (router, contracts) = ContractRouter::<()>::new()
            .get(
                "//contract/public//",
                OperationDescriptor::ContractPublic,
                AccessPolicy::Public,
                get(ok),
            )
            .expect("valid public contract")
            .into_parts();
        assert_eq!(contracts[0].path, "/contract/public");
        assert_eq!(contracts[0].access, RegisteredAccess::Public);
        let response = router
            .oneshot(Request::get("/contract/public").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[test]
    fn access_policies_are_canonical_and_reject_empty_collections_before_registration() {
        assert_eq!(
            normalize_access(AccessPolicy::Any(vec!["b", "a", "a"])).unwrap().0,
            RegisteredAccess::Any(vec!["a".into(), "b".into()])
        );
        assert_eq!(
            normalize_access(AccessPolicy::All(vec!["a"])).unwrap().0,
            RegisteredAccess::All(vec!["a".into()])
        );
        assert!(matches!(
            normalize_access(AccessPolicy::Any(vec![])),
            Err(ContractError::EmptyCapabilities)
        ));
        assert!(matches!(
            normalize_access(AccessPolicy::Require("")),
            Err(ContractError::EmptyCapability)
        ));

        for policy in [
            AccessPolicy::Require(""),
            AccessPolicy::Any(Vec::new()),
            AccessPolicy::All(Vec::new()),
        ] {
            assert!(
                ContractRouter::<()>::new()
                    .get("/rejected", OperationDescriptor::ContractAny, policy, get(ok))
                    .is_err()
            );
        }
    }

    #[test]
    fn duplicate_runtime_route_and_operation_id_are_rejected() {
        let duplicate_route = ContractRouter::<()>::new()
            .get("/same", OperationDescriptor::ContractPublic, AccessPolicy::Public, get(ok))
            .unwrap()
            .get("/same", OperationDescriptor::ContractAny, AccessPolicy::Public, get(ok));
        assert!(matches!(duplicate_route, Err(ContractError::RouteConflict { .. })));

        let duplicate_operation = ContractRouter::<()>::new()
            .get("/first", OperationDescriptor::ContractPublic, AccessPolicy::Public, get(ok))
            .unwrap()
            .get("/second", OperationDescriptor::ContractPublic, AccessPolicy::Public, get(ok));
        assert!(matches!(
            duplicate_operation,
            Err(ContractError::OperationIdConflict(operation_id))
                if operation_id == "contractPublic"
        ));
    }

    #[test]
    fn full_paths_are_composed_once() {
        assert_eq!(join_path("/api/", "/system/users/"), "/api/system/users");
        assert_eq!(join_path("/api", "/auth/me"), "/api/auth/me");
    }

    #[test]
    fn all_axum_http_methods_can_be_registered_with_one_contract_authority() {
        let router = ContractRouter::<()>::new()
            .get("/resource", OperationDescriptor::ContractPublic, AccessPolicy::Public, get(ok))
            .unwrap()
            .post(
                "/resource",
                OperationDescriptor::ContractAny,
                AccessPolicy::Public,
                axum::routing::post(ok),
            )
            .unwrap()
            .put(
                "/resource",
                OperationDescriptor::ContractAll,
                AccessPolicy::Public,
                axum::routing::put(ok),
            )
            .unwrap()
            .patch(
                "/resource",
                OperationDescriptor::BenchmarkRequire,
                AccessPolicy::Public,
                axum::routing::patch(ok),
            )
            .unwrap()
            .delete(
                "/resource",
                OperationDescriptor::CurrentAdminUser,
                AccessPolicy::Public,
                axum::routing::delete(ok),
            )
            .unwrap();

        let (_, contracts) = router.into_parts();
        assert_eq!(contracts.len(), 5);
        assert_eq!(
            contracts.iter().map(|contract| contract.method.clone()).collect::<Vec<_>>(),
            vec![Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE,]
        );
        assert!(contracts.iter().all(|contract| contract.path == "/resource"));
    }

    #[tokio::test]
    async fn public_any_and_all_enforce_their_real_router_policies() {
        use rustzen_auth::auth::CurrentUser;
        fn request(path: &str, capabilities: &[&str]) -> Request<Body> {
            let mut request = Request::get(path).body(Body::empty()).unwrap();
            request.extensions_mut().insert(CurrentUser::new(
                1,
                "test",
                capabilities.iter().map(|code| (*code).to_owned()),
                false,
            ));
            request
        }
        let (router, contracts) = ContractRouter::<()>::new()
            .get("/public", OperationDescriptor::ContractPublic, AccessPolicy::Public, get(ok))
            .unwrap()
            .get(
                "/any",
                OperationDescriptor::ContractAny,
                AccessPolicy::Any(vec!["a", "b"]),
                get(ok),
            )
            .unwrap()
            .get(
                "/all",
                OperationDescriptor::ContractAll,
                AccessPolicy::All(vec!["a", "b"]),
                get(ok),
            )
            .unwrap()
            .into_parts();
        assert_eq!(
            router
                .clone()
                .oneshot(Request::get("/public").body(Body::empty()).unwrap())
                .await
                .unwrap()
                .status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            router.clone().oneshot(request("/any", &["a"])).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            router.clone().oneshot(request("/all", &["a"])).await.unwrap().status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            router.clone().oneshot(request("/all", &["a", "b"])).await.unwrap().status(),
            StatusCode::NO_CONTENT
        );
        assert!(matches!(contracts[1].access, RegisteredAccess::Any(_)));
    }

    #[tokio::test]
    #[ignore = "run explicitly in release mode: measures isolated router registration and hot routing"]
    async fn route_contract_registration_and_hot_request_benchmark() {
        use rustzen_auth::{auth::CurrentUser, permission::RouterExt};
        use std::time::Instant;
        const WARMUP: usize = 2;
        const SAMPLES: usize = 9;
        const ITERATIONS: usize = 5_000;
        const CAPABILITY: &str = "benchmark:read";
        fn summary(mut values: Vec<f64>) -> (f64, f64) {
            values.sort_by(f64::total_cmp);
            (values[values.len() / 2], values[(values.len() * 95 / 100).min(values.len() - 1)])
        }

        let descriptors = [
            OperationDescriptor::Login,
            OperationDescriptor::CurrentAdminUser,
            OperationDescriptor::Logout,
            OperationDescriptor::UpdateAccountAvatar,
            OperationDescriptor::UpdateAccountProfile,
            OperationDescriptor::ChangeAccountPassword,
            OperationDescriptor::GetDashboardStats,
            OperationDescriptor::ListManageLogs,
            OperationDescriptor::ExportManageLogs,
            OperationDescriptor::ListManageTasks,
            OperationDescriptor::ListTaskRuns,
            OperationDescriptor::RunTask,
            OperationDescriptor::ListDeployments,
            OperationDescriptor::UploadDeployment,
            OperationDescriptor::CleanupDeployments,
            OperationDescriptor::GetDeployment,
            OperationDescriptor::ExpireDeployment,
            OperationDescriptor::DeleteDeployment,
            OperationDescriptor::DeployVersion,
            OperationDescriptor::ListMenus,
            OperationDescriptor::ListModuleMenuInventory,
            OperationDescriptor::UpdateMenu,
            OperationDescriptor::DeleteMenu,
            OperationDescriptor::GetMenuOptions,
            OperationDescriptor::ListRoles,
            OperationDescriptor::CreateRole,
            OperationDescriptor::UpdateRole,
            OperationDescriptor::DeleteRole,
            OperationDescriptor::GetRoleOptions,
            OperationDescriptor::GetStatusOverview,
            OperationDescriptor::ListModuleLogs,
            OperationDescriptor::TailModuleLog,
            OperationDescriptor::BackupModuleLogs,
            OperationDescriptor::PreviewModuleLogCleanup,
            OperationDescriptor::ConfirmModuleLogCleanup,
            OperationDescriptor::ListUsers,
            OperationDescriptor::CreateAdminUser,
            OperationDescriptor::UpdateUser,
            OperationDescriptor::DeleteUser,
            OperationDescriptor::GetUserOptions,
            OperationDescriptor::GetUserStatusOptions,
            OperationDescriptor::UpdateUserPassword,
            OperationDescriptor::UpdateUserStatus,
            OperationDescriptor::ListModules,
            OperationDescriptor::UpdateModuleEnabled,
            OperationDescriptor::GetModuleNavigation,
            OperationDescriptor::GetDashboardModules,
        ];
        assert_eq!(descriptors.len(), 47);

        fn method(index: usize) -> Method {
            match index % 5 {
                0 => Method::GET,
                1 => Method::POST,
                2 => Method::PUT,
                3 => Method::PATCH,
                _ => Method::DELETE,
            }
        }

        fn add_legacy(
            router: Router<()>,
            path: &str,
            method: Method,
            access: AccessPolicy,
        ) -> Router<()> {
            let permission = rustzen_auth::permission::PermissionsCheck::Require(CAPABILITY);
            match (method, access) {
                (Method::GET, AccessPolicy::Require(_)) => {
                    router.route_with_permission(path, get(ok), permission)
                }
                (Method::POST, AccessPolicy::Require(_)) => {
                    router.route_with_permission(path, axum::routing::post(ok), permission)
                }
                (Method::PUT, AccessPolicy::Require(_)) => {
                    router.route_with_permission(path, axum::routing::put(ok), permission)
                }
                (Method::PATCH, AccessPolicy::Require(_)) => {
                    router.route_with_permission(path, axum::routing::patch(ok), permission)
                }
                (Method::DELETE, AccessPolicy::Require(_)) => {
                    router.route_with_permission(path, axum::routing::delete(ok), permission)
                }
                (Method::GET, _) => router.route(path, get(ok)),
                (Method::POST, _) => router.route(path, axum::routing::post(ok)),
                (Method::PUT, _) => router.route(path, axum::routing::put(ok)),
                (Method::PATCH, _) => router.route(path, axum::routing::patch(ok)),
                (Method::DELETE, _) => router.route(path, axum::routing::delete(ok)),
                _ => router,
            }
        }

        fn add_contract(
            router: ContractRouter<()>,
            path: &str,
            method: Method,
            descriptor: OperationDescriptor,
            access: AccessPolicy,
        ) -> ContractRouter<()> {
            let result = match method {
                Method::GET => router.get(path, descriptor, access, get(ok)),
                Method::POST => router.post(path, descriptor, access, axum::routing::post(ok)),
                Method::PUT => router.put(path, descriptor, access, axum::routing::put(ok)),
                Method::PATCH => router.patch(path, descriptor, access, axum::routing::patch(ok)),
                Method::DELETE => {
                    router.delete(path, descriptor, access, axum::routing::delete(ok))
                }
                _ => unreachable!(),
            };
            result.expect("benchmark contract registration")
        }

        let mut legacy_register = Vec::new();
        let mut contract_register = Vec::new();
        for sample in 0..(WARMUP + SAMPLES) {
            let measure_legacy = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    let mut router = Router::<()>::new();
                    for (index, _) in descriptors.iter().enumerate() {
                        let access = if index < 2 {
                            AccessPolicy::Authenticated
                        } else {
                            AccessPolicy::Require(CAPABILITY)
                        };
                        router =
                            add_legacy(router, &format!("/bench/{index}"), method(index), access);
                    }
                }
                started.elapsed().as_nanos() as f64 / ITERATIONS as f64
            };
            let measure_contract = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    let mut router = ContractRouter::<()>::new();
                    for (index, descriptor) in descriptors.iter().enumerate() {
                        let access = if index == 0 {
                            AccessPolicy::Public
                        } else if index == 1 {
                            AccessPolicy::Authenticated
                        } else {
                            AccessPolicy::Require(CAPABILITY)
                        };
                        router = add_contract(
                            router,
                            &format!("/bench/{index}"),
                            method(index),
                            descriptor.clone(),
                            access,
                        );
                    }
                    let _ = router.into_parts();
                }
                started.elapsed().as_nanos() as f64 / ITERATIONS as f64
            };
            let (legacy, contract) = if sample % 2 == 0 {
                (measure_legacy(), measure_contract())
            } else {
                let contract = measure_contract();
                let legacy = measure_legacy();
                (legacy, contract)
            };
            if sample >= WARMUP {
                legacy_register.push(legacy);
                contract_register.push(contract);
            }
        }
        let legacy = Router::<()>::new()
            .route("/bench/public", get(ok))
            .route("/bench/auth", get(ok))
            .route_with_permission(
                "/bench/require",
                get(ok),
                rustzen_auth::permission::PermissionsCheck::Require(CAPABILITY),
            );
        let contract = ContractRouter::<()>::new()
            .get(
                "/bench/public",
                OperationDescriptor::ContractPublic,
                AccessPolicy::Public,
                get(ok),
            )
            .unwrap()
            .get(
                "/bench/auth",
                OperationDescriptor::CurrentAdminUser,
                AccessPolicy::Authenticated,
                get(ok),
            )
            .unwrap()
            .get(
                "/bench/require",
                OperationDescriptor::BenchmarkRequire,
                AccessPolicy::Require(CAPABILITY),
                get(ok),
            )
            .unwrap()
            .into_parts()
            .0;
        let mut legacy_hot = [Vec::new(), Vec::new(), Vec::new()];
        let mut contract_hot = [Vec::new(), Vec::new(), Vec::new()];
        fn request(path: &str, authenticated: bool) -> Request<Body> {
            let mut request = Request::get(path).body(Body::empty()).unwrap();
            if authenticated {
                request.extensions_mut().insert(CurrentUser::new(
                    1,
                    "benchmark",
                    [CAPABILITY.to_owned()],
                    false,
                ));
            }
            request
        }
        for sample in 0..(WARMUP + SAMPLES) {
            let measure_legacy = async {
                let mut values = Vec::new();
                for (index, (path, authenticated)) in
                    [("/bench/public", false), ("/bench/auth", true), ("/bench/require", true)]
                        .into_iter()
                        .enumerate()
                {
                    let started = Instant::now();
                    for _ in 0..ITERATIONS {
                        assert_eq!(
                            legacy
                                .clone()
                                .oneshot(request(path, authenticated))
                                .await
                                .unwrap()
                                .status(),
                            StatusCode::NO_CONTENT
                        );
                    }
                    values.push((index, started.elapsed().as_nanos() as f64 / ITERATIONS as f64));
                }
                values
            };
            let measure_contract = async {
                let mut values = Vec::new();
                for (index, (path, authenticated)) in
                    [("/bench/public", false), ("/bench/auth", true), ("/bench/require", true)]
                        .into_iter()
                        .enumerate()
                {
                    let started = Instant::now();
                    for _ in 0..ITERATIONS {
                        assert_eq!(
                            contract
                                .clone()
                                .oneshot(request(path, authenticated))
                                .await
                                .unwrap()
                                .status(),
                            StatusCode::NO_CONTENT
                        );
                    }
                    values.push((index, started.elapsed().as_nanos() as f64 / ITERATIONS as f64));
                }
                values
            };
            let (legacy_values, contract_values) = if sample % 2 == 0 {
                (measure_legacy.await, measure_contract.await)
            } else {
                let contract_ns = measure_contract.await;
                let legacy_ns = measure_legacy.await;
                (legacy_ns, contract_ns)
            };
            if sample >= WARMUP {
                for (index, value) in legacy_values {
                    legacy_hot[index].push(value);
                }
                for (index, value) in contract_values {
                    contract_hot[index].push(value);
                }
            }
        }
        let (lr50, lr95) = summary(legacy_register);
        let (cr50, cr95) = summary(contract_register);
        let hot_names = ["public", "authenticated", "require"];
        println!(
            "route-contract benchmark routes={} warmup={WARMUP} samples={SAMPLES} iterations={ITERATIONS} register_ns_op legacy_median={lr50:.1} legacy_p95={lr95:.1} contract_median={cr50:.1} contract_p95={cr95:.1} ratio={:.3}",
            descriptors.len(),
            cr50 / lr50
        );
        for (index, name) in hot_names.into_iter().enumerate() {
            let (legacy_median, legacy_p95) = summary(std::mem::take(&mut legacy_hot[index]));
            let (contract_median, contract_p95) = summary(std::mem::take(&mut contract_hot[index]));
            println!(
                "route-contract hot_access={name} legacy_median={legacy_median:.1} legacy_p95={legacy_p95:.1} contract_median={contract_median:.1} contract_p95={contract_p95:.1} ratio={:.3}",
                contract_median / legacy_median
            );
        }
    }
}
