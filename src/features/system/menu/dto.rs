use serde::Deserialize;

/// Create menu request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAndUpdateMenuDto {
    pub parent_id: i64,
    pub name: String,
    pub code: String,
    pub menu_type: i16,
    pub sort_order: i16,
    pub status: i16,
}

/// Menu query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuQueryDto {
    /// The name of the menu.
    pub name: Option<String>,
    /// The code of the menu.
    pub code: Option<String>,
    /// The status of the menu.
    pub status: Option<String>,
}
