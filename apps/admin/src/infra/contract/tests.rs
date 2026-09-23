use super::*;
use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode},
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

    for policy in
        [AccessPolicy::Require(""), AccessPolicy::Any(Vec::new()), AccessPolicy::All(Vec::new())]
    {
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
        .get("/any", OperationDescriptor::ContractAny, AccessPolicy::Any(vec!["a", "b"]), get(ok))
        .unwrap()
        .get("/all", OperationDescriptor::ContractAll, AccessPolicy::All(vec!["a", "b"]), get(ok))
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
        OperationDescriptor::RevokeUserSessions,
        OperationDescriptor::ListModules,
        OperationDescriptor::UpdateModuleEnabled,
        OperationDescriptor::GetModuleNavigation,
        OperationDescriptor::GetDashboardModules,
    ];
    assert_eq!(descriptors.len(), 48);

    fn method(index: usize) -> Method {
        match index % 5 {
            0 => Method::GET,
            1 => Method::POST,
            2 => Method::PUT,
            3 => Method::PATCH,
            _ => Method::DELETE,
        }
    }

    fn add_direct(
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
            Method::DELETE => router.delete(path, descriptor, access, axum::routing::delete(ok)),
            _ => unreachable!(),
        };
        result.expect("benchmark contract registration")
    }

    let mut direct_register = Vec::new();
    let mut contract_register = Vec::new();
    for sample in 0..(WARMUP + SAMPLES) {
        let measure_direct = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let mut router = Router::<()>::new();
                for (index, _) in descriptors.iter().enumerate() {
                    let access = if index < 2 {
                        AccessPolicy::Authenticated
                    } else {
                        AccessPolicy::Require(CAPABILITY)
                    };
                    router = add_direct(router, &format!("/bench/{index}"), method(index), access);
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
        let (direct, contract) = if sample % 2 == 0 {
            (measure_direct(), measure_contract())
        } else {
            let contract = measure_contract();
            let direct = measure_direct();
            (direct, contract)
        };
        if sample >= WARMUP {
            direct_register.push(direct);
            contract_register.push(contract);
        }
    }
    let direct = Router::<()>::new()
        .route("/bench/public", get(ok))
        .route("/bench/auth", get(ok))
        .route_with_permission(
            "/bench/require",
            get(ok),
            rustzen_auth::permission::PermissionsCheck::Require(CAPABILITY),
        );
    let contract = ContractRouter::<()>::new()
        .get("/bench/public", OperationDescriptor::ContractPublic, AccessPolicy::Public, get(ok))
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
    let mut direct_hot = [Vec::new(), Vec::new(), Vec::new()];
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
        let measure_direct = async {
            let mut values = Vec::new();
            for (index, (path, authenticated)) in
                [("/bench/public", false), ("/bench/auth", true), ("/bench/require", true)]
                    .into_iter()
                    .enumerate()
            {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    assert_eq!(
                        direct
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
        let (direct_values, contract_values) = if sample % 2 == 0 {
            (measure_direct.await, measure_contract.await)
        } else {
            let contract_ns = measure_contract.await;
            let direct_ns = measure_direct.await;
            (direct_ns, contract_ns)
        };
        if sample >= WARMUP {
            for (index, value) in direct_values {
                direct_hot[index].push(value);
            }
            for (index, value) in contract_values {
                contract_hot[index].push(value);
            }
        }
    }
    let (dr50, dr95) = summary(direct_register);
    let (cr50, cr95) = summary(contract_register);
    let hot_names = ["public", "authenticated", "require"];
    println!(
        "route-contract benchmark routes={} warmup={WARMUP} samples={SAMPLES} iterations={ITERATIONS} register_ns_op direct_median={dr50:.1} direct_p95={dr95:.1} contract_median={cr50:.1} contract_p95={cr95:.1} ratio={:.3}",
        descriptors.len(),
        cr50 / dr50
    );
    for (index, name) in hot_names.into_iter().enumerate() {
        let (direct_median, direct_p95) = summary(std::mem::take(&mut direct_hot[index]));
        let (contract_median, contract_p95) = summary(std::mem::take(&mut contract_hot[index]));
        println!(
            "route-contract hot_access={name} direct_median={direct_median:.1} direct_p95={direct_p95:.1} contract_median={contract_median:.1} contract_p95={contract_p95:.1} ratio={:.3}",
            contract_median / direct_median
        );
    }
}
