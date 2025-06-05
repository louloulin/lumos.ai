//! Enterprise-grade Authentication and Authorization System
//! 
//! This module provides comprehensive authentication and authorization capabilities
//! for Lumos.ai, including JWT/OAuth2 support, RBAC (Role-Based Access Control),
//! API key management, and multi-tenant security.
//! 
//! # Features
//! 
//! - **JWT Authentication**: Secure token-based authentication
//! - **OAuth2 Integration**: Support for external identity providers
//! - **RBAC System**: Fine-grained role and permission management
//! - **API Key Management**: Secure API key generation and validation
//! - **Multi-tenant Support**: Tenant isolation and resource management
//! - **Session Management**: Secure session handling and lifecycle
//! 
//! # Example
//! 
//! ```rust
//! use lumosai_core::auth::{AuthManager, JWTConfig, RBACManager};
//! 
//! // Initialize authentication manager
//! let auth_config = JWTConfig::new("your-secret-key");
//! let auth_manager = AuthManager::new(auth_config);
//! 
//! // Create user and assign roles
//! let user = auth_manager.create_user("user@example.com", "password").await?;
//! let rbac = RBACManager::new();
//! rbac.assign_role(&user.id, "agent_developer").await?;
//! 
//! // Validate permissions
//! let has_permission = rbac.check_permission(&user.id, "agents:create").await?;
//! ```

pub mod jwt;
pub mod oauth2;
pub mod rbac;
pub mod api_keys;
pub mod session;
pub mod multi_tenant;

#[cfg(test)]
mod integration_tests;

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Authentication and authorization errors
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    
    #[error("Token expired")]
    TokenExpired,
    
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("User not found: {0}")]
    UserNotFound(String),
    
    #[error("Role not found: {0}")]
    RoleNotFound(String),
    
    #[error("Tenant not found: {0}")]
    TenantNotFound(String),
    
    #[error("API key invalid or expired")]
    InvalidApiKey,
    
    #[error("Session expired")]
    SessionExpired,
    
    #[error("Authentication error: {0}")]
    Other(String),
}

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: Option<String>,
    pub tenant_id: Option<Uuid>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub created_at: SystemTime,
    pub last_login: Option<SystemTime>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

impl User {
    /// Create a new user
    pub fn new(email: String, tenant_id: Option<Uuid>) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            username: None,
            tenant_id,
            roles: Vec::new(),
            permissions: Vec::new(),
            created_at: SystemTime::now(),
            last_login: None,
            is_active: true,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if user has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(&permission.to_string())
    }
    
    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }
    
    /// Update last login time
    pub fn update_last_login(&mut self) {
        self.last_login = Some(SystemTime::now());
    }
}

/// Authentication token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub token_type: TokenType,
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub expires_at: SystemTime,
    pub scopes: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// Token types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenType {
    JWT,
    ApiKey,
    Session,
    OAuth2,
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: Duration,
    pub api_key_expiration: Duration,
    pub session_timeout: Duration,
    pub max_login_attempts: u32,
    pub lockout_duration: Duration,
    pub require_email_verification: bool,
    pub enable_multi_tenant: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "default-secret-change-in-production".to_string(),
            jwt_expiration: Duration::from_secs(24 * 3600), // 24 hours
            api_key_expiration: Duration::from_secs(90 * 24 * 3600), // 90 days
            session_timeout: Duration::from_secs(8 * 3600), // 8 hours
            max_login_attempts: 5,
            lockout_duration: Duration::from_secs(15 * 60), // 15 minutes
            require_email_verification: false,
            enable_multi_tenant: false,
        }
    }
}

/// Main authentication manager
#[derive(Debug)]
pub struct AuthManager {
    config: AuthConfig,
    jwt_manager: jwt::JWTManager,
    rbac_manager: rbac::RBACManager,
    api_key_manager: api_keys::ApiKeyManager,
    session_manager: session::SessionManager,
    tenant_manager: Option<multi_tenant::TenantManager>,
}

impl AuthManager {
    /// Create a new authentication manager
    pub fn new(config: AuthConfig) -> Self {
        let jwt_manager = jwt::JWTManager::new(config.jwt_secret.clone(), config.jwt_expiration);
        let rbac_manager = rbac::RBACManager::new();
        let api_key_manager = api_keys::ApiKeyManager::new(config.api_key_expiration);
        let session_manager = session::SessionManager::new(config.session_timeout);
        let tenant_manager = if config.enable_multi_tenant {
            Some(multi_tenant::TenantManager::new())
        } else {
            None
        };
        
        Self {
            config,
            jwt_manager,
            rbac_manager,
            api_key_manager,
            session_manager,
            tenant_manager,
        }
    }
    
    /// Authenticate user with email and password
    pub async fn authenticate(&self, email: &str, password: &str) -> AuthResult<AuthToken> {
        // TODO: Implement password verification
        // This would typically involve:
        // 1. Look up user by email
        // 2. Verify password hash
        // 3. Check if account is active
        // 4. Check login attempts and lockout
        // 5. Generate JWT token
        
        // For now, return a mock implementation
        let user = User::new(email.to_string(), None);
        self.jwt_manager.generate_token(&user).await
    }
    
    /// Validate authentication token
    pub async fn validate_token(&self, token: &str) -> AuthResult<User> {
        self.jwt_manager.validate_token(token).await
    }
    
    /// Create a new user
    pub async fn create_user(&self, email: &str, password: &str, tenant_id: Option<Uuid>) -> AuthResult<User> {
        // TODO: Implement user creation with password hashing
        let mut user = User::new(email.to_string(), tenant_id);
        
        // Note: In a real implementation, you would store the user in a database
        // and assign roles through the RBAC manager
        user.roles.push("user".to_string());
        
        Ok(user)
    }
    
    /// Generate API key for user (placeholder implementation)
    pub async fn generate_api_key(&self, user_id: Uuid, name: &str, _scopes: Vec<String>) -> AuthResult<String> {
        // In a real implementation, this would use a mutable reference or interior mutability
        Ok(format!("lum_{}_{}", user_id.to_string().replace('-', "")[..8].to_string(), name))
    }

    /// Validate API key (placeholder implementation)
    pub async fn validate_api_key(&self, api_key: &str) -> AuthResult<User> {
        // In a real implementation, this would look up the API key in storage
        if api_key.starts_with("lum_") {
            Ok(User::new("api-user@example.com".to_string(), None))
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }

    /// Check user permission (placeholder implementation)
    pub async fn check_permission(&self, _user_id: &Uuid, _permission: &str) -> AuthResult<bool> {
        // In a real implementation, this would use the RBAC manager
        Ok(true)
    }

    /// Assign role to user (placeholder implementation)
    pub async fn assign_role(&self, _user_id: &Uuid, _role: &str) -> AuthResult<()> {
        // In a real implementation, this would use the RBAC manager
        Ok(())
    }

    /// Remove role from user (placeholder implementation)
    pub async fn remove_role(&self, _user_id: &Uuid, _role: &str) -> AuthResult<()> {
        // In a real implementation, this would use the RBAC manager
        Ok(())
    }

    /// Create session for user (placeholder implementation)
    pub async fn create_session(&self, user_id: Uuid) -> AuthResult<String> {
        // In a real implementation, this would use the session manager
        Ok(format!("sess_{}", user_id.to_string().replace('-', "")))
    }

    /// Validate session (placeholder implementation)
    pub async fn validate_session(&self, session_id: &str) -> AuthResult<User> {
        // In a real implementation, this would use the session manager
        if session_id.starts_with("sess_") {
            Ok(User::new("session-user@example.com".to_string(), None))
        } else {
            Err(AuthError::SessionExpired)
        }
    }
    
    /// Logout user (invalidate session/token)
    pub async fn logout(&self, token: &str) -> AuthResult<()> {
        // TODO: Implement token blacklisting
        Ok(())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_auth_manager_creation() {
        let config = AuthConfig::default();
        let auth_manager = AuthManager::new(config);
        
        // Test that manager is created successfully
        assert!(!auth_manager.config.jwt_secret.is_empty());
    }
    
    #[tokio::test]
    async fn test_user_creation() {
        let user = User::new("test@example.com".to_string(), None);
        
        assert_eq!(user.email, "test@example.com");
        assert!(user.is_active);
        assert!(user.roles.is_empty());
        assert!(user.permissions.is_empty());
    }
    
    #[tokio::test]
    async fn test_user_permissions() {
        let mut user = User::new("test@example.com".to_string(), None);
        user.permissions.push("agents:create".to_string());
        user.roles.push("developer".to_string());
        
        assert!(user.has_permission("agents:create"));
        assert!(!user.has_permission("admin:delete"));
        assert!(user.has_role("developer"));
        assert!(!user.has_role("admin"));
    }
}
