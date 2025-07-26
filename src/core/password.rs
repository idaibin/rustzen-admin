use crate::common::error::ServiceError;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
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
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
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
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test_password_123";

        // Test hashing
        let hash = PasswordUtils::hash_password(password).expect("Should hash password");
        assert!(!hash.is_empty());

        // Test verification with correct password
        assert!(PasswordUtils::verify_password(password, &hash));

        // Test verification with incorrect password
        assert!(!PasswordUtils::verify_password("wrong_password", &hash));

        // Test verification with invalid hash
        assert!(!PasswordUtils::verify_password(password, "invalid_hash"));
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let password1 = "password1";
        let password2 = "password2";

        let hash1 = PasswordUtils::hash_password(password1).expect("Should hash password1");
        let hash2 = PasswordUtils::hash_password(password2).expect("Should hash password2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_produces_different_hashes() {
        // Due to random salt, same password should produce different hashes
        let password = "same_password";

        let hash1 = PasswordUtils::hash_password(password).expect("Should hash password");
        let hash2 = PasswordUtils::hash_password(password).expect("Should hash password");

        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(PasswordUtils::verify_password(password, &hash1));
        assert!(PasswordUtils::verify_password(password, &hash2));
    }
}
