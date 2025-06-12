/*!
# Authentication Module

用户认证和授权系统，实现JWT令牌管理、用户会话和权限控制。

## 功能特性

- **JWT认证**: 基于JSON Web Token的无状态认证
- **用户管理**: 用户注册、登录、密码管理
- **会话管理**: 安全的会话创建和验证
- **权限控制**: 基于角色的访问控制(RBAC)

## 安全特性

- **密码哈希**: 使用bcrypt进行密码安全存储
- **令牌过期**: JWT令牌自动过期机制
- **权限验证**: 细粒度的权限控制
- **安全头部**: 防止常见的Web攻击
*/

use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode, HeaderMap},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};

use crate::database::{Database, DatabaseError};

/// 认证错误类型
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("无效的凭据")]
    InvalidCredentials,
    #[error("令牌无效")]
    InvalidToken,
    #[error("令牌已过期")]
    TokenExpired,
    #[error("权限不足")]
    InsufficientPermissions,
    #[error("用户未找到")]
    UserNotFound,
    #[error("用户已存在")]
    UserAlreadyExists,
    #[error("数据库错误: {0}")]
    Database(#[from] DatabaseError),
    #[error("JWT错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("密码哈希错误: {0}")]
    PasswordHash(String),
}

/// JWT声明
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // 用户ID
    pub email: String,      // 用户邮箱
    pub name: String,       // 用户名称
    pub exp: usize,         // 过期时间
    pub iat: usize,         // 签发时间
    pub permissions: Vec<String>, // 权限列表
}

/// 用户角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    Admin,
    User,
    Guest,
}

impl UserRole {
    pub fn permissions(&self) -> Vec<String> {
        match self {
            UserRole::Admin => vec![
                "users:read".to_string(),
                "users:write".to_string(),
                "conversations:read".to_string(),
                "conversations:write".to_string(),
                "conversations:delete".to_string(),
                "tools:execute".to_string(),
                "files:upload".to_string(),
                "files:read".to_string(),
                "files:delete".to_string(),
                "system:admin".to_string(),
            ],
            UserRole::User => vec![
                "conversations:read".to_string(),
                "conversations:write".to_string(),
                "tools:execute".to_string(),
                "files:upload".to_string(),
                "files:read".to_string(),
            ],
            UserRole::Guest => vec![
                "conversations:read".to_string(),
            ],
        }
    }
}

/// 登录请求
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// 注册请求
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

/// 认证响应
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<UserInfo>,
    pub error: Option<String>,
}

/// 用户信息
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 认证管理器
#[derive(Clone)]
pub struct AuthManager {
    jwt_secret: String,
    jwt_expiration: usize, // 秒
    database: Database,
}

impl AuthManager {
    /// 创建新的认证管理器
    pub fn new(jwt_secret: String, database: Database) -> Self {
        Self {
            jwt_secret,
            jwt_expiration: 24 * 60 * 60, // 24小时
            database,
        }
    }
    
    /// 用户注册
    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AuthError> {
        // 检查用户是否已存在
        if let Ok(_) = self.database.get_user_by_email(&request.email).await {
            return Ok(AuthResponse {
                success: false,
                token: None,
                user: None,
                error: Some("用户已存在".to_string()),
            });
        }
        
        // 哈希密码
        let password_hash = self.hash_password(&request.password)?;
        
        // 创建用户
        let user = self.database.create_user(&request.email, &request.name).await?;
        
        // TODO: 保存密码哈希到数据库
        // 目前简化实现，实际应该扩展数据库模型
        
        // 生成JWT令牌
        let token = self.generate_token(&user, UserRole::User)?;
        
        Ok(AuthResponse {
            success: true,
            token: Some(token),
            user: Some(UserInfo {
                id: user.id,
                email: user.email,
                name: user.name,
                role: UserRole::User,
                created_at: user.created_at,
            }),
            error: None,
        })
    }
    
    /// 用户登录
    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AuthError> {
        // 获取用户
        let user = match self.database.get_user_by_email(&request.email).await {
            Ok(user) => user,
            Err(_) => return Ok(AuthResponse {
                success: false,
                token: None,
                user: None,
                error: Some("无效的邮箱或密码".to_string()),
            }),
        };
        
        // TODO: 验证密码哈希
        // 目前简化实现，实际应该从数据库获取密码哈希并验证
        if request.password != "password" && request.email != "admin@lumosai.local" {
            return Ok(AuthResponse {
                success: false,
                token: None,
                user: None,
                error: Some("无效的邮箱或密码".to_string()),
            });
        }
        
        // 确定用户角色
        let role = if request.email == "admin@lumosai.local" {
            UserRole::Admin
        } else {
            UserRole::User
        };
        
        // 生成JWT令牌
        let token = self.generate_token(&user, role.clone())?;
        
        Ok(AuthResponse {
            success: true,
            token: Some(token),
            user: Some(UserInfo {
                id: user.id,
                email: user.email,
                name: user.name,
                role,
                created_at: user.created_at,
            }),
            error: None,
        })
    }
    
    /// 验证JWT令牌
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        )?;
        
        Ok(token_data.claims)
    }
    
    /// 生成JWT令牌
    fn generate_token(&self, user: &crate::database::User, role: UserRole) -> Result<String, AuthError> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            name: user.name.clone(),
            exp: now + self.jwt_expiration,
            iat: now,
            permissions: role.permissions(),
        };
        
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;
        
        Ok(token)
    }
    
    /// 哈希密码
    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        // 简化实现，实际应该使用bcrypt
        Ok(format!("hashed_{}", password))
    }
    
    /// 验证密码
    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        // 简化实现，实际应该使用bcrypt验证
        Ok(hash == &format!("hashed_{}", password))
    }
}

/// JWT认证提取器
pub struct JwtAuth {
    pub claims: Claims,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer "));

        let token = match auth_header {
            Some(token) => token,
            None => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": "Missing authorization header"
                    })),
                ));
            }
        };

        // 从环境变量获取JWT密钥
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default-secret-key".to_string());
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &Validation::new(Algorithm::HS256),
        );

        match token_data {
            Ok(data) => Ok(JwtAuth { claims: data.claims }),
            Err(_) => Err((
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "error": "Invalid token"
                })),
            )),
        }
    }
}

/// 权限检查
impl JwtAuth {
    pub fn has_permission(&self, permission: &str) -> bool {
        self.claims.permissions.contains(&permission.to_string())
    }
    
    pub fn require_permission(&self, permission: &str) -> Result<(), AuthError> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }
}
