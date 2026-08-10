use axum::{
    Router,
    extract::{OriginalUri, Request, State},
    handler::Handler,
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing,
};

use crate::{
    DelegatedAccess, DelegationError, DelegationVerifier, ManifestError, ModuleDefinition,
    ModuleManifest, RouteManifest,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Require(pub &'static str);

pub struct ModuleRouter<S = ()> {
    module: String,
    verifier: DelegationVerifier,
    router: Router<S>,
    routes: Vec<RouteManifest>,
}

impl<S> ModuleRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn new(module: impl Into<String>, verifier: DelegationVerifier) -> Self {
        Self { module: module.into(), verifier, router: Router::new(), routes: Vec::new() }
    }

    pub fn get_with_permission<H, T>(
        self,
        path: &str,
        handler: H,
        required: Require,
    ) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.protected(
            Method::GET,
            path,
            routing::get(handler).head(|| async { StatusCode::METHOD_NOT_ALLOWED }),
            required,
        )
    }

    pub fn post_with_permission<H, T>(
        self,
        path: &str,
        handler: H,
        required: Require,
    ) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.protected(Method::POST, path, routing::post(handler), required)
    }

    pub fn put_with_permission<H, T>(
        self,
        path: &str,
        handler: H,
        required: Require,
    ) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.protected(Method::PUT, path, routing::put(handler), required)
    }

    pub fn patch_with_permission<H, T>(
        self,
        path: &str,
        handler: H,
        required: Require,
    ) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.protected(Method::PATCH, path, routing::patch(handler), required)
    }

    pub fn delete_with_permission<H, T>(
        self,
        path: &str,
        handler: H,
        required: Require,
    ) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.protected(Method::DELETE, path, routing::delete(handler), required)
    }

    pub fn post_public<H, T>(self, path: &str, handler: H) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.add_route(Method::POST, path, routing::post(handler), DelegatedAccess::Public)
    }

    pub fn options_public<H, T>(self, path: &str, handler: H) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.add_route(Method::OPTIONS, path, routing::options(handler), DelegatedAccess::Public)
    }

    pub fn get_public<H, T>(self, path: &str, handler: H) -> Result<Self, ManifestError>
    where
        H: Handler<T, S>,
        T: 'static,
    {
        self.add_route(
            Method::GET,
            path,
            routing::get(handler).head(|| async { StatusCode::METHOD_NOT_ALLOWED }),
            DelegatedAccess::Public,
        )
    }

    pub fn build(
        self,
        definition: &ModuleDefinition,
        release_version: impl Into<String>,
    ) -> Result<(Router<S>, ModuleManifest), ManifestError> {
        let manifest = definition.build_manifest(release_version, self.routes)?;
        Ok((self.router, manifest))
    }

    fn protected(
        self,
        method: Method,
        path: &str,
        method_router: axum::routing::MethodRouter<S>,
        required: Require,
    ) -> Result<Self, ManifestError> {
        self.add_route(method, path, method_router, DelegatedAccess::protected(required.0))
    }

    fn add_route(
        mut self,
        method: Method,
        path: &str,
        method_router: axum::routing::MethodRouter<S>,
        access: DelegatedAccess,
    ) -> Result<Self, ManifestError> {
        let route = match &access {
            DelegatedAccess::Protected(permission) => {
                RouteManifest::protected(method.clone(), path, permission)
            }
            DelegatedAccess::Public => RouteManifest::public(method.clone(), path),
        };
        let mut candidate_routes = self.routes.clone();
        candidate_routes.push(route.clone());
        crate::manifest::validate_routes(&self.module, &candidate_routes)?;
        let guard = RouteGuard {
            module: self.module.clone(),
            access: access.clone(),
            verifier: self.verifier.clone(),
        };
        self.router = self.router.route(
            path,
            method_router.route_layer(middleware::from_fn_with_state(guard, verify_delegation)),
        );
        self.routes.push(route);
        Ok(self)
    }
}

#[derive(Clone)]
struct RouteGuard {
    module: String,
    access: DelegatedAccess,
    verifier: DelegationVerifier,
}

async fn verify_delegation(
    State(guard): State<RouteGuard>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request
        .extensions()
        .get::<OriginalUri>()
        .map(|original| original.0.path())
        .unwrap_or_else(|| request.uri().path())
        .to_string();
    match guard.verifier.verify_for_route(
        request.headers(),
        request.method(),
        &path,
        &guard.module,
        &guard.access,
    ) {
        Ok(context) => {
            request.extensions_mut().insert(context);
            next.run(request).await
        }
        Err(error) => rejection(error),
    }
}

fn rejection(error: DelegationError) -> Response {
    let status = match error {
        DelegationError::CapabilityMismatch => StatusCode::FORBIDDEN,
        _ => StatusCode::UNAUTHORIZED,
    };
    (status, axum::Json(serde_json::json!({ "error": error.to_string() }))).into_response()
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{
        DelegatedAccess, DelegatedContext, DelegationSigner, DelegationVerifier, ModuleDefinition,
    };

    use super::{ModuleRouter, Require};

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

    #[tokio::test]
    async fn route_helpers_register_guard_and_manifest_metadata_once() {
        let verifier = DelegationVerifier::new("secret").expect("verifier");
        let module = ModuleRouter::<()>::new("reports", verifier)
            .get_with_permission("/jobs", || async { "jobs" }, Require("reports:view"))
            .expect("get route")
            .post_public("/track", || async { StatusCode::NO_CONTENT })
            .expect("public route")
            .options_public("/track", || async { StatusCode::NO_CONTENT })
            .expect("public options route");
        let definition = ModuleDefinition::from_toml(MODULE_TOML).expect("definition");
        let (app, manifest) = module.build(&definition, "0.5.0").expect("build");
        assert_eq!(manifest.routes.len(), 3);

        let context = DelegatedContext::new(
            "request-1",
            Some(7),
            "reports",
            Method::GET,
            "/jobs",
            DelegatedAccess::protected("reports:view"),
        )
        .expect("context");
        let headers =
            DelegationSigner::new("secret").expect("signer").sign(&context).expect("sign");
        let mut request = Request::builder().method(Method::GET).uri("/jobs");
        for (name, value) in &headers {
            request = request.header(name, value);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).expect("request"))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);

        let head_context = DelegatedContext::new(
            "request-2",
            Some(7),
            "reports",
            Method::HEAD,
            "/jobs",
            DelegatedAccess::protected("reports:view"),
        )
        .expect("head context");
        let head_headers =
            DelegationSigner::new("secret").expect("signer").sign(&head_context).expect("sign");
        let mut head_request = Request::builder().method(Method::HEAD).uri("/jobs");
        for (name, value) in &head_headers {
            head_request = head_request.header(name, value);
        }
        let head_response = app
            .clone()
            .oneshot(head_request.body(Body::empty()).expect("request"))
            .await
            .expect("response");
        assert_eq!(head_response.status(), StatusCode::METHOD_NOT_ALLOWED);

        let options_context = DelegatedContext::new(
            "request-3",
            None,
            "reports",
            Method::OPTIONS,
            "/track",
            DelegatedAccess::Public,
        )
        .expect("options context");
        let options_headers =
            DelegationSigner::new("secret").expect("signer").sign(&options_context).expect("sign");
        let mut options_request = Request::builder().method(Method::OPTIONS).uri("/track");
        for (name, value) in options_headers {
            if let Some(name) = name {
                options_request = options_request.header(name, value);
            }
        }
        let options_response = app
            .oneshot(options_request.body(Body::empty()).expect("request"))
            .await
            .expect("response");
        assert_eq!(options_response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn direct_unsigned_and_wrong_capability_requests_are_rejected() {
        let verifier = DelegationVerifier::new("secret").expect("verifier");
        let module = ModuleRouter::new("reports", verifier)
            .get_with_permission("/jobs", || async { "jobs" }, Require("reports:view"))
            .expect("get route");
        let definition = ModuleDefinition::from_toml(MODULE_TOML).expect("definition");
        let (app, _) = module.build(&definition, "0.5.0").expect("build");
        let unsigned = app
            .clone()
            .oneshot(Request::builder().uri("/jobs").body(Body::empty()).expect("request"))
            .await
            .expect("response");
        assert_eq!(unsigned.status(), StatusCode::UNAUTHORIZED);

        let context = DelegatedContext::new(
            "request-1",
            Some(7),
            "reports",
            Method::GET,
            "/jobs",
            DelegatedAccess::protected("reports:manage"),
        )
        .expect("context");
        let headers =
            DelegationSigner::new("secret").expect("signer").sign(&context).expect("sign");
        let mut request = Request::builder().uri("/jobs");
        for (name, value) in &headers {
            request = request.header(name, value);
        }
        let denied =
            app.oneshot(request.body(Body::empty()).expect("request")).await.expect("response");
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn ambiguous_routes_are_rejected_before_router_registration() {
        let verifier = DelegationVerifier::new("secret").expect("verifier");
        let module = ModuleRouter::<()>::new("reports", verifier)
            .get_with_permission("/jobs/{job_id}", || async { "job" }, Require("reports:view"))
            .expect("first route");
        assert!(
            module
                .get_with_permission("/jobs/{id}", || async { "job" }, Require("reports:view"),)
                .is_err()
        );
    }
}
