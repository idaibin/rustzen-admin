use serde::Deserialize;

/// Create and update role request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleDto {
    pub name: String,
    pub code: String,
    pub status: i16,
    pub menu_ids: Vec<i64>,
    pub description: Option<String>,
}

/// Update role request parameters

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleDto {
    pub name: String,
    pub code: String,
    pub status: i16,
    pub menu_ids: Vec<i64>,
    pub description: Option<String>,
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
    /// Filter by role code (case-insensitive search).
    pub role_code: Option<String>,
    /// Filter by role status.
    pub status: Option<String>,
}
