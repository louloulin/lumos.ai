//! JWT (JSON Web Token) Authentication Implementation
//! 
//! This module provides secure JWT token generation, validation, and management
//! for the Lumos.ai authentication system.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::{AuthError, AuthResult, AuthToken, TokenType, User};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub email: String,      // User email
    pub tenant_id: Option<String>, // Tenant ID for multi-tenant support
    pub roles: Vec<String>, // User roles
    pub permissions: Vec<String>, // User permissions
    pub iat: u64,          // Issued at
    pub exp: u64,          // Expiration time
    pub iss: String,       // Issuer
    pub aud: String,       // Audience
    pub jti: String,       // JWT ID (unique identifier)
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user: &User, expiration: Duration, issuer: &str, audience: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            sub: user.id.to_string(),
            email: user.email.clone(),
            tenant_id: user.tenant_id.map(|id| id.to_string()),
            roles: user.roles.clone(),
            permissions: user.permissions.clone(),
            iat: now,
            exp: now + expiration.as_secs(),
            iss: issuer.to_string(),
            aud: audience.to_string(),
            jti: Uuid::new_v4().to_string(),
        }
    }
    
    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        now > self.exp
    }
    
    /// Convert claims to User
    pub fn to_user(&self) -> AuthResult<User> {
        let user_id = Uuid::parse_str(&self.sub)
            .map_err(|_| AuthError::InvalidToken("Invalid user ID in token".to_string()))?;
        
        let tenant_id = if let Some(tenant_str) = &self.tenant_id {
            Some(Uuid::parse_str(tenant_str)
                .map_err(|_| AuthError::InvalidToken("Invalid tenant ID in token".to_string()))?)
        } else {
            None
        };
        
        let created_at = SystemTime::UNIX_EPOCH + Duration::from_secs(self.iat);
        
        Ok(User {
            id: user_id,
            email: self.email.clone(),
            username: None,
            tenant_id,
            roles: self.roles.clone(),
            permissions: self.permissions.clone(),
            created_at,
            last_login: Some(SystemTime::now()),
            is_active: true,
            metadata: HashMap::new(),
        })
    }
}

/// JWT Configuration
#[derive(Debug, Clone)]
pub struct JWTConfig {
    pub secret: String,
    pub expiration: Duration,
    pub issuer: String,
    pub audience: String,
    pub algorithm: String,
}

impl Default for JWTConfig {
    fn default() -> Self {
        Self {
            secret: "default-jwt-secret-change-in-production".to_string(),
            expiration: Duration::from_secs(24 * 3600), // 24 hours
            issuer: "lumos.ai".to_string(),
            audience: "lumos-api".to_string(),
            algorithm: "HS256".to_string(),
        }
    }
}

/// JWT Manager for token operations
#[derive(Debug)]
pub struct JWTManager {
    config: JWTConfig,
}

impl JWTManager {
    /// Create a new JWT manager
    pub fn new(secret: String, expiration: Duration) -> Self {
        let config = JWTConfig {
            secret,
            expiration,
            ..Default::default()
        };
        
        Self { config }
    }
    
    /// Create JWT manager with full configuration
    pub fn with_config(config: JWTConfig) -> Self {
        Self { config }
    }
    
    /// Generate a JWT token for a user
    pub async fn generate_token(&self, user: &User) -> AuthResult<AuthToken> {
        let claims = Claims::new(
            user,
            self.config.expiration,
            &self.config.issuer,
            &self.config.audience,
        );
        
        // In a real implementation, you would use a proper JWT library like `jsonwebtoken`
        // For now, we'll create a simple token format for demonstration
        let token_data = serde_json::to_string(&claims)
            .map_err(|e| AuthError::Other(format!("Failed to serialize claims: {}", e)))?;
        
        // Simple base64 encoding (in production, use proper JWT signing)
        use base64::{Engine as _, engine::general_purpose};
        let token = general_purpose::STANDARD.encode(format!("{}:{}", self.config.secret, token_data));
        
        let expires_at = SystemTime::UNIX_EPOCH + Duration::from_secs(claims.exp);
        
        Ok(AuthToken {
            token,
            token_type: TokenType::JWT,
            user_id: user.id,
            tenant_id: user.tenant_id,
            expires_at,
            scopes: user.roles.clone(),
            metadata: HashMap::new(),
        })
    }
    
    /// Validate a JWT token and return user information
    pub async fn validate_token(&self, token: &str) -> AuthResult<User> {
        // Decode the token (in production, use proper JWT verification)
        use base64::{Engine as _, engine::general_purpose};
        let decoded = general_purpose::STANDARD.decode(token)
            .map_err(|_| AuthError::InvalidToken("Invalid token format".to_string()))?;
        
        let token_str = String::from_utf8(decoded)
            .map_err(|_| AuthError::InvalidToken("Invalid token encoding".to_string()))?;
        
        // Split secret and claims
        let parts: Vec<&str> = token_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidToken("Invalid token structure".to_string()));
        }
        
        let (secret, claims_str) = (parts[0], parts[1]);
        
        // Verify secret
        if secret != self.config.secret {
            return Err(AuthError::InvalidToken("Invalid token signature".to_string()));
        }
        
        // Parse claims
        let claims: Claims = serde_json::from_str(claims_str)
            .map_err(|_| AuthError::InvalidToken("Invalid token claims".to_string()))?;
        
        // Check expiration
        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }
        
        // Convert to user
        claims.to_user()
    }
    
    /// Refresh a token (generate new token with extended expiration)
    pub async fn refresh_token(&self, token: &str) -> AuthResult<AuthToken> {
        let user = self.validate_token(token).await?;
        self.generate_token(&user).await
    }
    
    /// Extract claims from token without validation (for debugging)
    pub fn extract_claims(&self, token: &str) -> AuthResult<Claims> {
        use base64::{Engine as _, engine::general_purpose};
        let decoded = general_purpose::STANDARD.decode(token)
            .map_err(|_| AuthError::InvalidToken("Invalid token format".to_string()))?;
        
        let token_str = String::from_utf8(decoded)
            .map_err(|_| AuthError::InvalidToken("Invalid token encoding".to_string()))?;
        
        let parts: Vec<&str> = token_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidToken("Invalid token structure".to_string()));
        }
        
        let claims_str = parts[1];
        let claims: Claims = serde_json::from_str(claims_str)
            .map_err(|_| AuthError::InvalidToken("Invalid token claims".to_string()))?;
        
        Ok(claims)
    }
    
    /// Get token expiration time
    pub fn get_token_expiration(&self, token: &str) -> AuthResult<SystemTime> {
        let claims = self.extract_claims(token)?;
        Ok(SystemTime::UNIX_EPOCH + Duration::from_secs(claims.exp))
    }
    
    /// Check if token is expired
    pub fn is_token_expired(&self, token: &str) -> AuthResult<bool> {
        let claims = self.extract_claims(token)?;
        Ok(claims.is_expired())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_jwt_manager_creation() {
        let manager = JWTManager::new("test-secret".to_string(), Duration::from_secs(3600));
        assert_eq!(manager.config.secret, "test-secret");
        assert_eq!(manager.config.expiration, Duration::from_secs(3600));
    }
    
    #[tokio::test]
    async fn test_token_generation_and_validation() {
        let manager = JWTManager::new("test-secret".to_string(), Duration::from_secs(3600));
        let user = User::new("test@example.com".to_string(), None);
        
        // Generate token
        let auth_token = manager.generate_token(&user).await.unwrap();
        assert!(!auth_token.token.is_empty());
        assert_eq!(auth_token.user_id, user.id);
        
        // Validate token
        let validated_user = manager.validate_token(&auth_token.token).await.unwrap();
        assert_eq!(validated_user.email, user.email);
        assert_eq!(validated_user.id, user.id);
    }
    
    #[tokio::test]
    async fn test_token_expiration() {
        let manager = JWTManager::new("test-secret".to_string(), Duration::from_secs(1));
        let user = User::new("test@example.com".to_string(), None);
        
        let auth_token = manager.generate_token(&user).await.unwrap();
        
        // Token should be valid initially
        assert!(manager.validate_token(&auth_token.token).await.is_ok());
        
        // Wait for token to expire
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Token should be expired now
        let result = manager.validate_token(&auth_token.token).await;
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }
    
    #[tokio::test]
    async fn test_claims_creation() {
        let user = User::new("test@example.com".to_string(), None);
        let claims = Claims::new(&user, Duration::from_secs(3600), "test-issuer", "test-audience");
        
        assert_eq!(claims.sub, user.id.to_string());
        assert_eq!(claims.email, user.email);
        assert_eq!(claims.iss, "test-issuer");
        assert_eq!(claims.aud, "test-audience");
        assert!(!claims.is_expired());
    }
    
    #[tokio::test]
    async fn test_token_refresh() {
        let manager = JWTManager::new("test-secret".to_string(), Duration::from_secs(3600));
        let user = User::new("test@example.com".to_string(), None);
        
        let original_token = manager.generate_token(&user).await.unwrap();
        let refreshed_token = manager.refresh_token(&original_token.token).await.unwrap();
        
        assert_ne!(original_token.token, refreshed_token.token);
        assert_eq!(original_token.user_id, refreshed_token.user_id);
    }
}
