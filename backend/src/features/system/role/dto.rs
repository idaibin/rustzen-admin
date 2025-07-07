use serde::Deserialize;

/// Create role request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleDto {
    pub role_name: String,
    pub role_code: String,
    pub description: Option<String>,
    pub status: Option<i16>,
    pub menu_ids: Vec<i64>,
}

/// Update role request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleDto {
    pub role_name: Option<String>,
    pub role_code: Option<String>,
    pub description: Option<String>,
    pub status: Option<i16>,
    pub menu_ids: Option<Vec<i64>>,
}

/// Role query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleQueryDto {
    /// The page number to retrieve. Defaults to 1.
    pub current: Option<i64>,
    /// The number of items per page. Defaults to 10.
    pub page_size: Option<i64>,
    /// Filter by role name (case-insensitive search).
    pub role_name: Option<String>,
    /// Filter by role status.
    pub status: Option<i16>,
}
