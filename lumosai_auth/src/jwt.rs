//! JWT Token 生成和验证
//!
//! 提供基于 jsonwebtoken 的真实 JWT 实现，替换之前的假实现

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{AuthError, Result};

/// JWT Claims（令牌声明）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User email
    pub email: String,
    /// User roles
    pub roles: Vec<String>,
    /// Expiration time (unix timestamp)
    pub exp: usize,
    /// Issued at (unix timestamp)
    pub iat: usize,
    /// Issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// JWT ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
}

impl Claims {
    /// Create new claims
    pub fn new(user_id: String, email: String, roles: Vec<String>, expires_in_secs: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::seconds(expires_in_secs))
            .timestamp() as usize;

        Self {
            sub: user_id,
            email,
            roles,
            exp,
            iat: now.timestamp() as usize,
            iss: Some("lumosai".to_string()),
            jti: Some(uuid::Uuid::new_v4().to_string()),
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp() as usize;
        self.exp < now
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> i64 {
        let now = Utc::now().timestamp() as usize;
        if self.exp > now {
            (self.exp - now) as i64
        } else {
            0
        }
    }
}

/// JWT 认证管理器
#[derive(Clone)]
pub struct JwtAuth {
    encoding_key: Arc<EncodingKey>,
    decoding_key: Arc<DecodingKey>,
    pub(crate) expiration_secs: i64,
    issuer: String,
}

impl JwtAuth {
    /// Create new JWT auth manager
    ///
    /// # Arguments
    ///
    /// * `secret` - Secret key for signing tokens (should be at least 32 bytes)
    /// * `expiration_secs` - Token expiration time in seconds (default: 3600 = 1 hour)
    ///
    /// # Examples
    ///
    /// ```
    /// use lumosai_auth::jwt::JwtAuth;
    ///
    /// let auth = JwtAuth::new("your-secret-key-at-least-32-bytes", 3600);
    /// ```
    pub fn new(secret: &str, expiration_secs: i64) -> Self {
        Self {
            encoding_key: Arc::new(EncodingKey::from_secret(secret.as_bytes())),
            decoding_key: Arc::new(DecodingKey::from_secret(secret.as_bytes())),
            expiration_secs,
            issuer: "lumosai".to_string(),
        }
    }

    /// Create with default expiration (1 hour)
    pub fn with_default_expiration(secret: &str) -> Self {
        Self::new(secret, 3600)
    }

    /// Generate JWT token
    ///
    /// # Arguments
    ///
    /// * `user_id` - Unique user identifier
    /// * `email` - User email
    /// * `roles` - User roles (e.g., ["admin", "user"])
    ///
    /// # Returns
    ///
    /// Signed JWT token string
    ///
    /// # Examples
    ///
    /// ```
    /// use lumosai_auth::jwt::JwtAuth;
    ///
    /// # tokio_test::block_on(async {
    /// let auth = JwtAuth::new("secret", 3600);
    /// let token = auth.generate_token(
    ///     "user123",
    ///     "user@example.com",
    ///     vec!["user".to_string()]
    /// ).unwrap();
    /// # });
    /// ```
    pub fn generate_token(
        &self,
        user_id: &str,
        email: &str,
        roles: Vec<String>,
    ) -> Result<String> {
        let claims = Claims::new(
            user_id.to_string(),
            email.to_string(),
            roles,
            self.expiration_secs,
        );

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            AuthError::AuthenticationFailed(format!("Failed to generate token: {}", e))
        })
    }

    /// Verify and decode JWT token
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string to verify
    ///
    /// # Returns
    ///
    /// Decoded claims if token is valid
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Token signature is invalid
    /// - Token is expired
    /// - Token format is invalid
    ///
    /// # Examples
    ///
    /// ```
    /// use lumosai_auth::jwt::JwtAuth;
    ///
    /// # tokio_test::block_on(async {
    /// let auth = JwtAuth::new("secret", 3600);
    /// let token = auth.generate_token("user123", "user@example.com", vec![]).unwrap();
    ///
    /// let claims = auth.verify_token(&token).unwrap();
    /// assert_eq!(claims.sub, "user123");
    /// # });
    /// ```
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AuthError::InvalidToken(format!("Token validation failed: {}", e)))?;

        let claims = token_data.claims;

        // Check if token is expired
        if claims.is_expired() {
            return Err(AuthError::SessionExpired);
        }

        Ok(claims)
    }

    /// Refresh token
    ///
    /// Generates a new token with the same claims but updated expiration
    ///
    /// # Arguments
    ///
    /// * `old_token` - Existing token to refresh
    ///
    /// # Returns
    ///
    /// New JWT token
    pub fn refresh_token(&self, old_token: &str) -> Result<String> {
        // Verify old token (allow expired tokens for refresh)
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);
        validation.validate_exp = false; // Allow expired tokens

        let claims = decode::<Claims>(old_token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::InvalidToken(format!("Invalid token: {}", e)))?;

        // Generate new token with same info
        self.generate_token(&claims.sub, &claims.email, claims.roles)
    }

    /// Validate token and extract user info
    ///
    /// Convenience method that verifies token and returns user information
    pub fn validate_and_get_user(&self, token: &str) -> Result<(String, String, Vec<String>)> {
        let claims = self.verify_token(token)?;
        Ok((claims.sub, claims.email, claims.roles))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new(
            "user123".to_string(),
            "test@example.com".to_string(),
            vec!["user".to_string()],
            3600,
        );

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.roles, vec!["user".to_string()]);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_jwt_auth_creation() {
        let auth = JwtAuth::new("test-secret-key", 3600);
        assert_eq!(auth.expiration_secs, 3600);
    }

    #[test]
    fn test_generate_and_verify_token() {
        let auth = JwtAuth::new("test-secret-key-at-least-32-bytes-long", 3600);

        let token = auth
            .generate_token("user123", "test@example.com", vec!["user".to_string()])
            .unwrap();

        assert!(!token.is_empty());
        assert!(token.contains('.'));  // JWT has dots

        let claims = auth.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
        assert_eq!(claims.roles, vec!["user".to_string()]);
    }

    #[test]
    fn test_token_expiration() {
        let auth = JwtAuth::new("test-secret", -1); // Expired immediately

        let token = auth
            .generate_token("user123", "test@example.com", vec![])
            .unwrap();

        // Token should be expired
        let result = auth.verify_token(&token);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::SessionExpired));
    }

    #[test]
    fn test_invalid_token() {
        let auth = JwtAuth::new("test-secret", 3600);

        let result = auth.verify_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret() {
        let auth1 = JwtAuth::new("secret1", 3600);
        let auth2 = JwtAuth::new("secret2", 3600);

        let token = auth1
            .generate_token("user123", "test@example.com", vec![])
            .unwrap();

        // Verifying with different secret should fail
        let result = auth2.verify_token(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_token() {
        let auth = JwtAuth::new("test-secret-key", 3600);

        let token = auth
            .generate_token("user123", "test@example.com", vec!["user".to_string()])
            .unwrap();

        let new_token = auth.refresh_token(&token).unwrap();
        assert_ne!(token, new_token);

        let claims = auth.verify_token(&new_token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, "test@example.com");
    }

    #[test]
    fn test_validate_and_get_user() {
        let auth = JwtAuth::new("test-secret-key", 3600);

        let token = auth
            .generate_token("user123", "test@example.com", vec!["admin".to_string()])
            .unwrap();

        let (user_id, email, roles) = auth.validate_and_get_user(&token).unwrap();
        assert_eq!(user_id, "user123");
        assert_eq!(email, "test@example.com");
        assert_eq!(roles, vec!["admin".to_string()]);
    }
}

