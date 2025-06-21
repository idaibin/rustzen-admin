// Menu-related data structures (database models, API request/response bodies) go here.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Menu {
    pub id: i32,
    pub parent_id: i32,
    pub name: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub r#type: i32, // 0: Directory, 1: Menu, 2: Button
}
