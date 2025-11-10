//! 用户管理和数据结构
//!
//! 定义用户相关的数据结构和管理功能

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// 用户唯一标识
    pub id: String,
    /// 用户邮箱
    pub email: String,
    /// 用户角色列表
    pub roles: Vec<String>,
    /// 租户 ID（多租户支持）
    pub tenant_id: Option<String>,
    /// 用户创建时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    /// 最后登录时间
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTime<Utc>>,
    /// 用户状态
    pub status: UserStatus,
    /// 自定义元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// 用户状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserStatus {
    /// 活跃状态
    Active,
    /// 禁用状态
    Disabled,
    /// 待激活
    Pending,
    /// 已删除
    Deleted,
}

impl User {
    /// Create a new user
    pub fn new(id: String, email: String) -> Self {
        Self {
            id,
            email,
            roles: vec!["user".to_string()],
            tenant_id: None,
            created_at: Some(Utc::now()),
            last_login: None,
            status: UserStatus::Active,
            metadata: None,
        }
    }

    /// Create a new user with roles
    pub fn with_roles(id: String, email: String, roles: Vec<String>) -> Self {
        Self {
            id,
            email,
            roles,
            tenant_id: None,
            created_at: Some(Utc::now()),
            last_login: None,
            status: UserStatus::Active,
            metadata: None,
        }
    }

    /// Add a role to the user
    pub fn add_role(&mut self, role: String) {
        if !self.roles.contains(&role) {
            self.roles.push(role);
        }
    }

    /// Remove a role from the user
    pub fn remove_role(&mut self, role: &str) {
        self.roles.retain(|r| r != role);
    }

    /// Check if user has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all of the specified roles
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }

    /// Update last login time
    pub fn update_last_login(&mut self) {
        self.last_login = Some(Utc::now());
    }
}

/// 用户凭据（用于注册和登录）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub email: String,
    pub password: String,
}

impl UserCredentials {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

/// 用户注册请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub roles: Option<Vec<String>>,
    pub tenant_id: Option<String>,
}

/// 用户登录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// 用户登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
    pub expires_in: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new("user123".to_string(), "test@example.com".to_string());

        assert_eq!(user.id, "user123");
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.roles, vec!["user".to_string()]);
        assert_eq!(user.status, UserStatus::Active);
        assert!(user.created_at.is_some());
    }

    #[test]
    fn test_user_with_roles() {
        let user = User::with_roles(
            "admin123".to_string(),
            "admin@example.com".to_string(),
            vec!["admin".to_string(), "user".to_string()],
        );

        assert_eq!(user.roles.len(), 2);
        assert!(user.has_role("admin"));
        assert!(user.has_role("user"));
    }

    #[test]
    fn test_add_role() {
        let mut user = User::new("user123".to_string(), "test@example.com".to_string());

        user.add_role("admin".to_string());
        assert!(user.has_role("admin"));
        assert_eq!(user.roles.len(), 2);

        // Adding same role again shouldn't duplicate
        user.add_role("admin".to_string());
        assert_eq!(user.roles.len(), 2);
    }

    #[test]
    fn test_remove_role() {
        let mut user = User::with_roles(
            "user123".to_string(),
            "test@example.com".to_string(),
            vec!["admin".to_string(), "user".to_string()],
        );

        user.remove_role("admin");
        assert!(!user.has_role("admin"));
        assert!(user.has_role("user"));
    }

    #[test]
    fn test_has_any_role() {
        let user = User::with_roles(
            "user123".to_string(),
            "test@example.com".to_string(),
            vec!["editor".to_string()],
        );

        assert!(user.has_any_role(&["admin", "editor"]));
        assert!(!user.has_any_role(&["admin", "superuser"]));
    }

    #[test]
    fn test_has_all_roles() {
        let user = User::with_roles(
            "user123".to_string(),
            "test@example.com".to_string(),
            vec!["admin".to_string(), "user".to_string()],
        );

        assert!(user.has_all_roles(&["admin", "user"]));
        assert!(!user.has_all_roles(&["admin", "user", "superuser"]));
    }

    #[test]
    fn test_is_active() {
        let mut user = User::new("user123".to_string(), "test@example.com".to_string());

        assert!(user.is_active());

        user.status = UserStatus::Disabled;
        assert!(!user.is_active());
    }

    #[test]
    fn test_update_last_login() {
        let mut user = User::new("user123".to_string(), "test@example.com".to_string());

        assert!(user.last_login.is_none());

        user.update_last_login();
        assert!(user.last_login.is_some());
    }
}

