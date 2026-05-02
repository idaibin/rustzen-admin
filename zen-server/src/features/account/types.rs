use serde::Deserialize;

/// Request payload for current-account profile updates.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountProfileRequest {
    pub email: String,
    pub real_name: Option<String>,
}

/// Request payload for current-account password changes.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeAccountPasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PasswordHashRow {
    pub password_hash: String,
}
