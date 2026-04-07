use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthClaims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}
