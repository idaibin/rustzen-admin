// Log-related data structures (database models, API request/response bodies) go here.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: i64,
    pub level: String,
    pub message: String,
    pub created_at: String,
}
