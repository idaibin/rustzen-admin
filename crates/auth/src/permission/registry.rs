use std::sync::RwLock;

use once_cell::sync::Lazy;
use tracing::info;

static ROUTE_PERMISSION_CODES: Lazy<RwLock<Vec<String>>> = Lazy::new(|| RwLock::new(Vec::new()));

pub fn register_permission_codes<I>(codes: I)
where
    I: IntoIterator<Item = &'static str>,
{
    let mut registry = ROUTE_PERMISSION_CODES.write().expect("permission registry lock poisoned");
    let count = registry.len();
    registry.extend(codes.into_iter().map(ToString::to_string));
    info!("Registered {} permission codes (total: {})", registry.len() - count, registry.len());
}

pub fn take_registered_permission_codes() -> Vec<String> {
    let mut registry = ROUTE_PERMISSION_CODES.write().expect("permission registry lock poisoned");
    let codes = std::mem::take(&mut *registry);
    info!("Took {} registered permission codes", codes.len());
    codes
}
