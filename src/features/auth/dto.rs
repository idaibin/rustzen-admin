use serde::Deserialize;

/// Request payload for user authentication.
#[derive(Deserialize)]
pub struct LoginRequest {
    /// Username or email for authentication
    pub username: String,
    /// User's password in plain text
    pub password: String,
}
