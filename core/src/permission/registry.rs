use std::sync::RwLock;

use once_cell::sync::Lazy;

static ROUTE_PERMISSION_CODES: Lazy<RwLock<Vec<String>>> = Lazy::new(|| RwLock::new(Vec::new()));

pub fn register_permission_codes<I>(codes: I)
where
    I: IntoIterator<Item = &'static str>,
{
    let mut registry = ROUTE_PERMISSION_CODES.write().expect("permission registry lock poisoned");
    registry.extend(codes.into_iter().map(ToString::to_string));
}

pub fn take_registered_permission_codes() -> Vec<String> {
    let mut registry = ROUTE_PERMISSION_CODES.write().expect("permission registry lock poisoned");
    std::mem::take(&mut *registry)
}
