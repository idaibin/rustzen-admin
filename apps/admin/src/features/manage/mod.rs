pub mod deploy;
pub mod log;
pub mod task;

use crate::infra::contract::ContractRouter;
use sqlx::SqlitePool;

use deploy::deploy_routes;
use log::log_routes;
use task::task_routes;

pub fn manage_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .nest("/logs", log_routes())
        .expect("static manage contract")
        .nest("/tasks", task_routes())
        .expect("static manage contract")
        .nest("/deploy", deploy_routes())
        .expect("static manage contract")
}
