//! 密码哈希和验证
//!
//! 使用 Argon2 进行密码哈希（比 bcrypt 更安全）

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

use crate::{AuthError, Result};

/// 密码哈希管理器
///
/// 使用 Argon2id 算法进行密码哈希，这是当前最安全的密码哈希算法
pub struct PasswordHashManager;

impl PasswordHashManager {
    /// Hash a password using Argon2
    ///
    /// # Arguments
    ///
    /// * `password` - Plain text password to hash
    ///
    /// # Returns
    ///
    /// PHC string format hash (can be stored in database)
    ///
    /// # Examples
    ///
    /// ```
    /// use lumosai_auth::password::PasswordHashManager;
    ///
    /// let hash = PasswordHashManager::hash_password("mypassword").unwrap();
    /// assert!(hash.starts_with("$argon2"));
    /// ```
    pub fn hash_password(password: &str) -> Result<String> {
        if password.is_empty() {
            return Err(AuthError::AuthenticationFailed(
                "Password cannot be empty".to_string(),
            ));
        }

        if password.len() < 8 {
            return Err(AuthError::AuthenticationFailed(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| {
                AuthError::AuthenticationFailed(format!("Failed to hash password: {}", e))
            })
    }

    /// Verify a password against its hash
    ///
    /// # Arguments
    ///
    /// * `password` - Plain text password to verify
    /// * `hash` - Previously hashed password (PHC string format)
    ///
    /// # Returns
    ///
    /// `true` if password matches hash, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use lumosai_auth::password::PasswordHashManager;
    ///
    /// let hash = PasswordHashManager::hash_password("mypassword").unwrap();
    /// let is_valid = PasswordHashManager::verify_password("mypassword", &hash).unwrap();
    /// assert!(is_valid);
    ///
    /// let is_invalid = PasswordHashManager::verify_password("wrongpassword", &hash).unwrap();
    /// assert!(!is_invalid);
    /// ```
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        if password.is_empty() || hash.is_empty() {
            return Ok(false);
        }

        let parsed_hash = PasswordHash::new(hash).map_err(|e| {
            AuthError::AuthenticationFailed(format!("Invalid hash format: {}", e))
        })?;

        let argon2 = Argon2::default();

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Validate password strength
    ///
    /// # Requirements
    ///
    /// - At least 8 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one digit
    ///
    /// # Examples
    ///
    /// ```
    /// use lumosai_auth::password::PasswordHashManager;
    ///
    /// assert!(PasswordHashManager::validate_password_strength("Password123").is_ok());
    /// assert!(PasswordHashManager::validate_password_strength("weak").is_err());
    /// ```
    pub fn validate_password_strength(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(AuthError::AuthenticationFailed(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());

        if !has_uppercase {
            return Err(AuthError::AuthenticationFailed(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if !has_lowercase {
            return Err(AuthError::AuthenticationFailed(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if !has_digit {
            return Err(AuthError::AuthenticationFailed(
                "Password must contain at least one digit".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if hash needs rehashing (e.g., if algorithm parameters changed)
    pub fn needs_rehash(hash: &str) -> bool {
        // For now, always return false as we use current Argon2 defaults
        // In production, you might want to check if hash uses old parameters
        let _ = hash;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let hash = PasswordHashManager::hash_password("TestPassword123").unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_different_each_time() {
        let hash1 = PasswordHashManager::hash_password("TestPassword123").unwrap();
        let hash2 = PasswordHashManager::hash_password("TestPassword123").unwrap();

        // Same password should produce different hashes (due to different salts)
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_password_success() {
        let password = "TestPassword123";
        let hash = PasswordHashManager::hash_password(password).unwrap();

        let is_valid = PasswordHashManager::verify_password(password, &hash).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_password_failure() {
        let hash = PasswordHashManager::hash_password("TestPassword123").unwrap();

        let is_valid = PasswordHashManager::verify_password("WrongPassword", &hash).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_empty_password() {
        let result = PasswordHashManager::hash_password("");
        assert!(result.is_err());
    }

    #[test]
    fn test_short_password() {
        let result = PasswordHashManager::hash_password("short");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_strength_validation() {
        // Valid password
        assert!(PasswordHashManager::validate_password_strength("Password123").is_ok());

        // Too short
        assert!(PasswordHashManager::validate_password_strength("Pass1").is_err());

        // No uppercase
        assert!(PasswordHashManager::validate_password_strength("password123").is_err());

        // No lowercase
        assert!(PasswordHashManager::validate_password_strength("PASSWORD123").is_err());

        // No digit
        assert!(PasswordHashManager::validate_password_strength("Password").is_err());
    }

    #[test]
    fn test_verify_empty_inputs() {
        let result = PasswordHashManager::verify_password("", "hash");
        assert_eq!(result.unwrap(), false);

        let result = PasswordHashManager::verify_password("password", "");
        assert_eq!(result.unwrap(), false);
    }
}

