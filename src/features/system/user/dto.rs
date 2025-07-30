use serde::Deserialize;

/// Create user request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserDto {
    pub username: String,
    pub email: String,
    pub password: String,
    pub real_name: Option<String>,
    /// User status: Defaults to 1.
    pub status: Option<i16>,
    /// A list of role IDs to assign to the user. If empty, will use default role.
    #[serde(default)]
    pub role_ids: Vec<i64>,
}

/// Update user request parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserDto {
    pub email: String,
    pub real_name: String,
    /// A list of role IDs to assign to the user. If provided, replaces all existing roles.
    pub role_ids: Vec<i64>,
}

/// User query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQueryDto {
    /// The page number to retrieve. Defaults to 1.
    pub current: Option<i64>,
    /// The number of items per page. Defaults to 10.
    pub page_size: Option<i64>,
    /// Filter by username (case-insensitive search).
    pub username: Option<String>,
    /// Filter by user status. Accepts: "normal"/"1", "disabled"/"2", or "all".
    pub status: Option<String>,
    /// Filter by real name (case-insensitive search).
    pub real_name: Option<String>,
    /// Filter by email (case-insensitive search).
    pub email: Option<String>,
}

/// User options query parameters
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOptionsDto {
    /// Search keyword
    pub q: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<i64>,
    /// Filter by user status
    pub status: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserPasswordDto {
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateUserStatusDto {
    pub status: i16,
}
