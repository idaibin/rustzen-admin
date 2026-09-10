mod routes;
mod runtime;

#[cfg(any(feature = "full", feature = "selected-distribution", test))]
pub(crate) use routes::documented_all_contracts;
#[cfg(test)]
pub(crate) use routes::documented_protected_routes;
pub use runtime::run_server;

#[cfg(all(test, feature = "selected-distribution"))]
mod monitor_distribution_tests;
#[cfg(all(test, feature = "full"))]
mod tests;

#[cfg(feature = "selected-distribution")]
mod selected_contract;
#[cfg(feature = "selected-distribution")]
pub(crate) use selected_contract::selected_contract_json;
