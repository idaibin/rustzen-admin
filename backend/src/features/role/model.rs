// Role-related data structures (database models, API request/response bodies) go here.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: i32,
    pub role_name: String,
    pub role_code: String,
    pub remark: Option<String>,
}
