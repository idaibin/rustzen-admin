// Dictionary-related data structures (database models, API request/response bodies) go here.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItem {
    pub id: i32,
    pub dict_type: String,
    pub label: String,
    pub value: String,
    pub is_default: bool,
}
