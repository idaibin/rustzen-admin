use serde::Deserialize;

/// Create dictionary item request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDictDto {
    /// The type of the dictionary, used to group related items (e.g., "user_status").
    pub dict_type: String,
    /// The display text for the item (e.g., "Active").
    pub label: String,
    /// The actual value of the item (e.g., "1").
    pub value: String,
    /// Whether this is the default item of its type.
    pub is_default: Option<bool>,
}

/// Update dictionary item request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDictDto {
    /// The type of the dictionary, used to group related items (e.g., "user_status").
    pub dict_type: Option<String>,
    /// The display text for the item (e.g., "Active").
    pub label: Option<String>,
    /// The actual value of the item (e.g., "1").
    pub value: Option<String>,
    /// Whether this is the default item of its type.
    pub is_default: Option<bool>,
}

/// Dictionary query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictQueryDto {
    /// Filter by dictionary type.
    pub dict_type: Option<String>,
    /// Search term for filtering labels.
    pub q: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<i64>,
}
