//! LumosAI 认证和授权模块
//!
//! 提供真正的 JWT 认证、密码哈希、用户管理等功能
//!
//! # 主要功能
//!
//! - **JWT 认证**: 真正的 JWT token 生成和验证
//! - **密码哈希**: Argon2 密码哈希（比 bcrypt 更安全）
//! - **用户管理**: 用户数据结构和角色管理
//!
//! # 快速开始
//!
//! ```
//! use lumosai_auth::{JwtAuth, PasswordHashManager, User};
//!
//! # tokio_test::block_on(async {
//! // 1. 创建 JWT 管理器
//! let jwt_auth = JwtAuth::new("your-secret-key-at-least-32-bytes", 3600);
//!
//! // 2. 哈希密码
//! let password_hash = PasswordHashManager::hash_password("MyPassword123").unwrap();
//!
//! // 3. 生成 JWT token
//! let token = jwt_auth.generate_token(
//!     "user123",
//!     "user@example.com",
//!     vec!["user".to_string()]
//! ).unwrap();
//!
//! // 4. 验证 token
//! let claims = jwt_auth.verify_token(&token).unwrap();
//! assert_eq!(claims.sub, "user123");
//! # });
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub mod jwt;
pub mod password;
pub mod user;

pub use jwt::{Claims, JwtAuth};
pub use password::PasswordHashManager;
pub use user::{LoginRequest, LoginResponse, RegisterRequest, User, UserCredentials, UserStatus};

/// 认证错误类型
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("认证失败: {0}")]
    AuthenticationFailed(String),

    #[error("授权失败: {0}")]
    AuthorizationFailed(String),

    #[error("令牌无效: {0}")]
    InvalidToken(String),

    #[error("会话过期")]
    SessionExpired,

    #[error("权限不足")]
    InsufficientPermissions,

    #[error("用户不存在")]
    UserNotFound,

    #[error("用户已存在")]
    UserAlreadyExists,
}

pub type Result<T> = std::result::Result<T, AuthError>;

/// 认证令牌
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub token_type: String,
    pub expires_in: u64,
}

/// 真正的认证服务（使用 JWT）
///
/// 替换之前的假实现，使用真正的 JWT token 和 Argon2 密码哈希
pub struct AuthService {
    jwt_auth: Arc<JwtAuth>,
    // 简单的内存存储（生产环境应使用数据库）
    users: Arc<RwLock<HashMap<String, (User, String)>>>, // (User, password_hash)
}

impl AuthService {
    /// Create new auth service with JWT
    ///
    /// # Arguments
    ///
    /// * `secret_key` - Secret key for JWT signing (at least 32 bytes recommended)
    /// * `expiration_secs` - Token expiration time in seconds
    ///
    /// # Examples
    ///
    /// ```
    /// use lumosai_auth::AuthService;
    ///
    /// let auth = AuthService::new("my-secret-key-at-least-32-bytes-long".to_string(), 3600);
    /// ```
    pub fn new(secret_key: String, expiration_secs: i64) -> Self {
        Self {
            jwt_auth: Arc::new(JwtAuth::new(&secret_key, expiration_secs)),
            users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default expiration (1 hour)
    pub fn with_default_expiration(secret_key: String) -> Self {
        Self::new(secret_key, 3600)
    }

    /// Register a new user
    ///
    /// # Arguments
    ///
    /// * `email` - User email
    /// * `password` - Plain text password (will be hashed)
    /// * `roles` - User roles (optional, defaults to ["user"])
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        roles: Option<Vec<String>>,
    ) -> Result<User> {
        // Validate password strength
        PasswordHashManager::validate_password_strength(password)?;

        // Check if user already exists
        {
            let users = self.users.read().unwrap();
            if users.values().any(|(u, _)| u.email == email) {
                return Err(AuthError::UserAlreadyExists);
            }
        }

        // Hash password
        let password_hash = PasswordHashManager::hash_password(password)?;

        // Create user
        let user_id = uuid::Uuid::new_v4().to_string();
        let user_roles = roles.unwrap_or_else(|| vec!["user".to_string()]);
        let user = User::with_roles(user_id.clone(), email.to_string(), user_roles);

        // Store user
        {
            let mut users = self.users.write().unwrap();
            users.insert(user_id, (user.clone(), password_hash));
        }

        Ok(user)
    }

    /// Authenticate user and return JWT token
    ///
    /// # Arguments
    ///
    /// * `email` - User email
    /// * `password` - Plain text password
    ///
    /// # Returns
    ///
    /// `AuthToken` containing JWT token
    pub async fn authenticate(&self, email: &str, password: &str) -> Result<AuthToken> {
        if email.is_empty() || password.is_empty() {
            return Err(AuthError::AuthenticationFailed(
                "Invalid credentials".to_string(),
            ));
        }

        // Find user by email
        let (user, password_hash) = {
            let users = self.users.read().unwrap();
            users
                .values()
                .find(|(u, _)| u.email == email)
                .cloned()
                .ok_or_else(|| AuthError::UserNotFound)?
        };

        // Check if user is active
        if !user.is_active() {
            return Err(AuthError::AuthenticationFailed(
                "User account is disabled".to_string(),
            ));
        }

        // Verify password
        let is_valid = PasswordHashManager::verify_password(password, &password_hash)?;
        if !is_valid {
            return Err(AuthError::AuthenticationFailed(
                "Invalid credentials".to_string(),
            ));
        }

        // Generate JWT token
        let token = self
            .jwt_auth
            .generate_token(&user.id, &user.email, user.roles.clone())?;

        // Update last login
        {
            let mut users = self.users.write().unwrap();
            if let Some((mut user, _)) = users.get_mut(&user.id).cloned() {
                user.update_last_login();
                users.insert(user.id.clone(), (user, password_hash));
            }
        }

        Ok(AuthToken {
            token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_auth.expiration_secs as u64,
        })
    }

    /// Validate JWT token and return user
    ///
    /// # Arguments
    ///
    /// * `token` - JWT token string
    ///
    /// # Returns
    ///
    /// `User` if token is valid
    pub async fn validate_token(&self, token: &str) -> Result<User> {
        if token.is_empty() {
            return Err(AuthError::InvalidToken("Empty token".to_string()));
        }

        // Verify JWT token
        let claims = self.jwt_auth.verify_token(token)?;

        // Get user from storage
        let users = self.users.read().unwrap();
        let (user, _) = users
            .get(&claims.sub)
            .ok_or_else(|| AuthError::UserNotFound)?;

        // Check if user is still active
        if !user.is_active() {
            return Err(AuthError::AuthorizationFailed(
                "User account is disabled".to_string(),
            ));
        }

        Ok(user.clone())
    }

    /// Refresh JWT token
    pub async fn refresh_token(&self, old_token: &str) -> Result<AuthToken> {
        let new_token = self.jwt_auth.refresh_token(old_token)?;

        Ok(AuthToken {
            token: new_token,
            token_type: "Bearer".to_string(),
            expires_in: self.jwt_auth.expiration_secs as u64,
        })
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> Result<User> {
        let users = self.users.read().unwrap();
        let (user, _) = users
            .get(user_id)
            .ok_or_else(|| AuthError::UserNotFound)?;
        Ok(user.clone())
    }

    /// Check if user has required role
    pub async fn check_permission(&self, user_id: &str, required_role: &str) -> Result<bool> {
        let user = self.get_user(user_id).await?;
        Ok(user.has_role(required_role))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_register_and_authenticate() {
        let auth = AuthService::with_default_expiration("test-secret-key-32-bytes-long!".to_string());

        // Register user
        let user = auth
            .register("test@example.com", "Password123", None)
            .await
            .unwrap();

        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.roles, vec!["user".to_string()]);

        // Authenticate
        let token_result = auth.authenticate("test@example.com", "Password123").await;
        assert!(token_result.is_ok());

        let auth_token = token_result.unwrap();
        assert!(!auth_token.token.is_empty());
        assert_eq!(auth_token.token_type, "Bearer");
        assert_eq!(auth_token.expires_in, 3600);

        // Token should contain dots (JWT format)
        assert!(auth_token.token.contains('.'));
    }

    #[tokio::test]
    async fn test_validate_token() {
        let auth = AuthService::with_default_expiration("test-secret-key".to_string());

        // Register and authenticate
        auth.register("test@example.com", "Password123", None)
            .await
            .unwrap();

        let auth_token = auth
            .authenticate("test@example.com", "Password123")
            .await
            .unwrap();

        // Validate token
        let user = auth.validate_token(&auth_token.token).await.unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let auth = AuthService::with_default_expiration("test-secret-key".to_string());

        auth.register("test@example.com", "Password123", None)
            .await
            .unwrap();

        let result = auth.authenticate("test@example.com", "WrongPassword").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_user_not_found() {
        let auth = AuthService::with_default_expiration("test-secret-key".to_string());

        let result = auth
            .authenticate("nonexistent@example.com", "Password123")
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AuthError::UserNotFound));
    }

    #[tokio::test]
    async fn test_duplicate_registration() {
        let auth = AuthService::with_default_expiration("test-secret-key".to_string());

        auth.register("test@example.com", "Password123", None)
            .await
            .unwrap();

        let result = auth.register("test@example.com", "Password456", None).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AuthError::UserAlreadyExists
        ));
    }

    #[tokio::test]
    async fn test_refresh_token() {
        let auth = AuthService::with_default_expiration("test-secret-key".to_string());

        auth.register("test@example.com", "Password123", None)
            .await
            .unwrap();

        let auth_token = auth
            .authenticate("test@example.com", "Password123")
            .await
            .unwrap();

        let refreshed = auth.refresh_token(&auth_token.token).await.unwrap();
        assert_ne!(auth_token.token, refreshed.token);

        // New token should be valid
        let user = auth.validate_token(&refreshed.token).await.unwrap();
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_check_permission() {
        let auth = AuthService::with_default_expiration("test-secret-key".to_string());

        let user = auth
            .register("admin@example.com", "Password123", Some(vec!["admin".to_string()]))
            .await
            .unwrap();

        let has_admin = auth.check_permission(&user.id, "admin").await.unwrap();
        assert!(has_admin);

        let has_superuser = auth.check_permission(&user.id, "superuser").await.unwrap();
        assert!(!has_superuser);
    }
}
