use std::collections::BTreeSet;

use axum::{Router, http::Method, routing::MethodRouter};
use rustzen_auth::permission::{PermissionsCheck, RouterExt};

use super::{AccessPolicy, ContractError, OperationDescriptor, RegisteredAccess, RouteContract};

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

pub(super) fn normalize_access(
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

pub(super) fn join_path(prefix: &str, path: &str) -> String {
    normalize_path(&format!("{}/{}", prefix.trim_matches('/'), path.trim_matches('/')))
}

fn normalize_path(path: &str) -> String {
    let segments = path.split('/').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
    if segments.is_empty() { "/".to_owned() } else { format!("/{}", segments.join("/")) }
}
