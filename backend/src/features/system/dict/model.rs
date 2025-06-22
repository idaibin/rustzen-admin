use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItem {
    pub id: i64,
    pub dict_type: String,
    pub label: String,
    pub value: String,
    pub is_default: bool,
}
