use crate::common::error::ServiceError;

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

/// Password utilities for secure hashing and verification.
pub struct PasswordUtils;

impl PasswordUtils {
    /// Hashes a plain-text password using Argon2.
    ///
    /// This function generates a random salt and uses Argon2 with default parameters
    /// to create a secure hash of the provided password.
    ///
    /// # Arguments
    ///
    /// * `password` - The plain-text password to hash
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The hashed password as a string
    /// * `Err(ServiceError::PasswordHashingFailed)` - If hashing fails
    pub fn hash_password(password: &str) -> Result<String, ServiceError> {
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes())
            .map_err(|_| ServiceError::PasswordHashingFailed)?
            .to_string();
        Ok(password_hash)
    }

    /// Verifies a password against a hash.
    ///
    /// This function parses the stored hash and verifies if the provided
    /// plain-text password matches the hash.
    ///
    /// # Arguments
    ///
    /// * `password` - The plain-text password to verify
    /// * `hash` - The stored hash to verify against
    ///
    /// # Returns
    ///
    /// * `true` - If the password matches the hash
    /// * `false` - If the password doesn't match or hash parsing fails
    pub fn verify_password(password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
}

#[cfg(test)]
#[path = "password_tests.rs"]
mod tests;
