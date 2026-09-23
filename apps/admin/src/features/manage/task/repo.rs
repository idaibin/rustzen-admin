//! Persistence composition for the bounded Admin maintenance catalog.
//!
//! Task summaries and run lifecycle writes are intentionally separate so this
//! composition remains the only persistence entry point used by the scheduler.

mod run_rows;
mod task_rows;

use sqlx::SqlitePool;

pub(super) use run_rows::{FinishTaskRunInput, InsertTaskRunInput};
pub(super) use task_rows::SyncTaskInput;

pub struct TaskRepository {
    pub(super) pool: SqlitePool,
}

impl TaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

fn map_db_error(err: sqlx::Error) -> crate::common::error::ServiceError {
    tracing::error!(?err, "Task database error");
    crate::common::error::ServiceError::DatabaseQueryFailed
}
