mod routes;
mod runtime;

pub(crate) use routes::documented_all_contracts;
#[cfg(test)]
pub(crate) use routes::documented_protected_routes;
pub use runtime::run_server;

#[cfg(test)]
mod tests;
