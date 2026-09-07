use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthClaims {
    pub iss: String,
    pub aud: String,
    pub sid: String,
    pub user_id: i64,
    pub username: String,
    pub user_auth_epoch: i64,
    pub exp: usize,
    pub iat: usize,
}
