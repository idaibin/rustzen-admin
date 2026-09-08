mod routes;
mod runtime;

#[cfg(any(feature = "full", feature = "monitor-distribution", test))]
pub(crate) use routes::documented_all_contracts;
#[cfg(test)]
pub(crate) use routes::documented_protected_routes;
pub use runtime::run_server;

#[cfg(all(test, feature = "monitor-distribution"))]
mod monitor_distribution_tests;
#[cfg(all(test, feature = "full"))]
mod tests;

#[cfg(feature = "monitor-distribution")]
mod selected_contract;
#[cfg(feature = "monitor-distribution")]
pub(crate) use selected_contract::selected_contract_json;
