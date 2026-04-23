use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

use super::AuthClaims;

#[derive(Debug, Clone)]
pub struct JwtCodec {
    secret: String,
    expiration_seconds: i64,
}

impl JwtCodec {
    pub fn new(secret: impl Into<String>, expiration_seconds: i64) -> Self {
        Self { secret: secret.into(), expiration_seconds }
    }

    pub fn encode(
        &self,
        user_id: i64,
        username: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let claims = AuthClaims {
            user_id,
            username: username.to_string(),
            exp: (now + Duration::seconds(self.expiration_seconds)).timestamp() as usize,
            iat: now.timestamp() as usize,
        };
        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
    }

    pub fn decode(&self, token: &str) -> Result<AuthClaims, jsonwebtoken::errors::Error> {
        let validation = Validation::new(Algorithm::HS256);
        let token = decode::<AuthClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;
        Ok(token.claims)
    }
}
