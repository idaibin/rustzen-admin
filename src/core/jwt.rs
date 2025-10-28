use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing;

use crate::core::config::CONFIG;

/// JWT Configuration, loaded from environment variables.
///
/// This struct holds the essential settings for JWT generation and validation,
/// including the secret key and token expiration duration.
#[derive(Debug)]
pub struct JwtConfig {
    /// The secret key used for signing and verifying tokens.
    pub secret: String,
    /// The duration in seconds for which a token is valid.
    pub expiration: i64,
}

/// The global JWT configuration instance, initialized lazily.
///
/// Reads `JWT_SECRET` and `JWT_EXPIRATION` from environment variables.
/// Provides default values and logs warnings if they are not set, which is
/// crucial for security and debugging during development.
pub static JWT_CONFIG: Lazy<JwtConfig> = Lazy::new(|| {
    let secret = CONFIG.jwt_secret.clone();
    let expiration = CONFIG.jwt_expiration;

    tracing::info!("JWT_CONFIG: {:?}", JwtConfig { secret: secret.clone(), expiration });
    JwtConfig { secret, expiration }
});

/// Represents the claims in the JWT payload.
///
/// These claims contain the token's subject information and metadata.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// The subject of the token, typically the user ID.
    pub user_id: i64,
    /// The username associated with the token.
    pub username: String,
    /// Expiration time (as a Unix timestamp).
    pub exp: usize,
    /// Issued at time (as a Unix timestamp).
    pub iat: usize,
}

/// Generates a new JWT for a given user.
///
/// # Arguments
///
/// * `user_id` - The ID of the user for whom the token is generated.
/// * `username` - The username of the user.
///
/// # Errors
///
/// Returns a `jsonwebtoken::errors::Error` if token generation fails.
pub fn generate_token(user_id: i64, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(JWT_CONFIG.expiration)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims { user_id, username: username.to_string(), exp, iat };

    tracing::debug!("Generating token for user '{}' (ID: {})", username, user_id);

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_CONFIG.secret.as_bytes()))
}

/// Verifies a JWT and returns the claims if valid.
///
/// # Arguments
///
/// * `token` - The JWT string to verify.
///
/// # Errors
///
/// Returns a `jsonwebtoken::errors::Error` if the token is invalid, expired,
/// or if verification otherwise fails.
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_CONFIG.secret.as_bytes()),
        &validation,
    )?;

    tracing::trace!("Successfully verified token for user '{}'", token_data.claims.username);
    Ok(token_data.claims)
}
