use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

use super::AuthClaims;

const DEFAULT_ISSUER: &str = "rustzen-admin";
const DEFAULT_AUDIENCE: &str = "rustzen-entry";

#[derive(Debug, Clone)]
pub struct JwtCodec {
    secret: String,
    expiration_seconds: i64,
    issuer: String,
    audience: String,
}

impl JwtCodec {
    pub fn new(secret: impl Into<String>, expiration_seconds: i64) -> Self {
        Self::with_issuer_audience(secret, expiration_seconds, DEFAULT_ISSUER, DEFAULT_AUDIENCE)
    }

    pub fn with_issuer_audience(
        secret: impl Into<String>,
        expiration_seconds: i64,
        issuer: impl Into<String>,
        audience: impl Into<String>,
    ) -> Self {
        Self {
            secret: secret.into(),
            expiration_seconds,
            issuer: issuer.into(),
            audience: audience.into(),
        }
    }

    pub fn encode(
        &self,
        user_id: i64,
        username: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        self.encode_session(user_id, username, &uuid::Uuid::new_v4().to_string(), 1)
    }

    pub fn encode_session(
        &self,
        user_id: i64,
        username: &str,
        sid: &str,
        user_auth_epoch: i64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        self.encode_claims(&self.claims_at(
            user_id,
            username,
            sid,
            user_auth_epoch,
            Utc::now().timestamp(),
        ))
    }

    pub fn claims_at(
        &self,
        user_id: i64,
        username: &str,
        sid: &str,
        user_auth_epoch: i64,
        issued_at: i64,
    ) -> AuthClaims {
        AuthClaims {
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
            sid: sid.to_string(),
            user_id,
            username: username.to_string(),
            user_auth_epoch,
            exp: (issued_at + Duration::seconds(self.expiration_seconds).num_seconds()) as usize,
            iat: issued_at as usize,
        }
    }

    pub fn encode_claims(
        &self,
        claims: &AuthClaims,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))
    }

    pub fn decode(&self, token: &str) -> Result<AuthClaims, jsonwebtoken::errors::Error> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 0;
        validation.set_audience(&[self.audience.as_str()]);
        validation.set_issuer(&[self.issuer.as_str()]);
        validation.set_required_spec_claims(&[
            "exp",
            "iat",
            "iss",
            "aud",
            "sid",
            "user_auth_epoch",
            "user_id",
            "username",
        ]);
        let token = decode::<AuthClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;
        Ok(token.claims)
    }
}
