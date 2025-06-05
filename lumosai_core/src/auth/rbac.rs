//! Role-Based Access Control (RBAC) System
//! 
//! This module implements a comprehensive RBAC system for fine-grained
//! permission management in Lumos.ai.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::{AuthError, AuthResult};

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
    pub conditions: Vec<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(name: String, description: String, resource: String, action: String) -> Self {
        Self {
            name,
            description,
            resource,
            action,
            conditions: Vec::new(),
        }
    }
    
    /// Create permission with conditions
    pub fn with_conditions(mut self, conditions: Vec<String>) -> Self {
        self.conditions = conditions;
        self
    }
    
    /// Check if permission matches a resource and action
    pub fn matches(&self, resource: &str, action: &str) -> bool {
        (self.resource == "*" || self.resource == resource) &&
        (self.action == "*" || self.action == action)
    }
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: HashSet<String>,
    pub parent_roles: HashSet<String>,
    pub is_system_role: bool,
    pub metadata: HashMap<String, String>,
}

impl Role {
    /// Create a new role
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system_role: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a system role (cannot be deleted)
    pub fn system_role(name: String, description: String) -> Self {
        Self {
            name,
            description,
            permissions: HashSet::new(),
            parent_roles: HashSet::new(),
            is_system_role: true,
            metadata: HashMap::new(),
        }
    }
    
    /// Add permission to role
    pub fn add_permission(&mut self, permission: String) {
        self.permissions.insert(permission);
    }
    
    /// Remove permission from role
    pub fn remove_permission(&mut self, permission: &str) {
        self.permissions.remove(permission);
    }
    
    /// Add parent role (for role inheritance)
    pub fn add_parent_role(&mut self, parent_role: String) {
        self.parent_roles.insert(parent_role);
    }
    
    /// Check if role has a specific permission
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.contains(permission)
    }
}

/// User role assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub user_id: Uuid,
    pub role_name: String,
    pub assigned_by: Uuid,
    pub assigned_at: std::time::SystemTime,
    pub expires_at: Option<std::time::SystemTime>,
    pub conditions: Vec<String>,
}

/// RBAC Manager for role and permission management
#[derive(Debug)]
pub struct RBACManager {
    roles: HashMap<String, Role>,
    permissions: HashMap<String, Permission>,
    user_roles: HashMap<Uuid, HashSet<String>>,
    role_cache: HashMap<Uuid, HashSet<String>>, // Cached effective permissions per user
}

impl RBACManager {
    /// Create a new RBAC manager with default roles and permissions
    pub fn new() -> Self {
        let mut manager = Self {
            roles: HashMap::new(),
            permissions: HashMap::new(),
            user_roles: HashMap::new(),
            role_cache: HashMap::new(),
        };
        
        // Initialize default roles and permissions
        manager.initialize_defaults();
        manager
    }
    
    /// Initialize default roles and permissions
    fn initialize_defaults(&mut self) {
        // Default permissions
        let permissions = vec![
            Permission::new("agents:create".to_string(), "Create agents".to_string(), "agents".to_string(), "create".to_string()),
            Permission::new("agents:read".to_string(), "Read agents".to_string(), "agents".to_string(), "read".to_string()),
            Permission::new("agents:update".to_string(), "Update agents".to_string(), "agents".to_string(), "update".to_string()),
            Permission::new("agents:delete".to_string(), "Delete agents".to_string(), "agents".to_string(), "delete".to_string()),
            Permission::new("tools:create".to_string(), "Create tools".to_string(), "tools".to_string(), "create".to_string()),
            Permission::new("tools:read".to_string(), "Read tools".to_string(), "tools".to_string(), "read".to_string()),
            Permission::new("tools:update".to_string(), "Update tools".to_string(), "tools".to_string(), "update".to_string()),
            Permission::new("tools:delete".to_string(), "Delete tools".to_string(), "tools".to_string(), "delete".to_string()),
            Permission::new("workflows:create".to_string(), "Create workflows".to_string(), "workflows".to_string(), "create".to_string()),
            Permission::new("workflows:read".to_string(), "Read workflows".to_string(), "workflows".to_string(), "read".to_string()),
            Permission::new("workflows:execute".to_string(), "Execute workflows".to_string(), "workflows".to_string(), "execute".to_string()),
            Permission::new("admin:*".to_string(), "Full admin access".to_string(), "*".to_string(), "*".to_string()),
        ];
        
        for permission in permissions {
            self.permissions.insert(permission.name.clone(), permission);
        }
        
        // Default roles
        let mut user_role = Role::system_role("user".to_string(), "Basic user role".to_string());
        user_role.add_permission("agents:read".to_string());
        user_role.add_permission("tools:read".to_string());
        user_role.add_permission("workflows:read".to_string());
        
        let mut developer_role = Role::system_role("developer".to_string(), "Developer role".to_string());
        developer_role.add_permission("agents:create".to_string());
        developer_role.add_permission("agents:read".to_string());
        developer_role.add_permission("agents:update".to_string());
        developer_role.add_permission("tools:create".to_string());
        developer_role.add_permission("tools:read".to_string());
        developer_role.add_permission("tools:update".to_string());
        developer_role.add_permission("workflows:create".to_string());
        developer_role.add_permission("workflows:read".to_string());
        developer_role.add_permission("workflows:execute".to_string());
        developer_role.add_parent_role("user".to_string());
        
        let mut admin_role = Role::system_role("admin".to_string(), "Administrator role".to_string());
        admin_role.add_permission("admin:*".to_string());
        admin_role.add_parent_role("developer".to_string());
        
        self.roles.insert("user".to_string(), user_role);
        self.roles.insert("developer".to_string(), developer_role);
        self.roles.insert("admin".to_string(), admin_role);
    }
    
    /// Create a new role
    pub async fn create_role(&mut self, name: String, description: String) -> AuthResult<()> {
        if self.roles.contains_key(&name) {
            return Err(AuthError::Other(format!("Role '{}' already exists", name)));
        }
        
        let role = Role::new(name.clone(), description);
        self.roles.insert(name, role);
        Ok(())
    }
    
    /// Delete a role (cannot delete system roles)
    pub async fn delete_role(&mut self, name: &str) -> AuthResult<()> {
        let role = self.roles.get(name)
            .ok_or_else(|| AuthError::RoleNotFound(name.to_string()))?;
        
        if role.is_system_role {
            return Err(AuthError::Other("Cannot delete system role".to_string()));
        }
        
        self.roles.remove(name);
        
        // Remove role from all users
        for user_roles in self.user_roles.values_mut() {
            user_roles.remove(name);
        }
        
        // Clear cache
        self.role_cache.clear();
        
        Ok(())
    }
    
    /// Add permission to role
    pub async fn add_permission_to_role(&mut self, role_name: &str, permission: &str) -> AuthResult<()> {
        let role = self.roles.get_mut(role_name)
            .ok_or_else(|| AuthError::RoleNotFound(role_name.to_string()))?;
        
        if !self.permissions.contains_key(permission) {
            return Err(AuthError::Other(format!("Permission '{}' does not exist", permission)));
        }
        
        role.add_permission(permission.to_string());
        self.role_cache.clear(); // Clear cache
        Ok(())
    }
    
    /// Remove permission from role
    pub async fn remove_permission_from_role(&mut self, role_name: &str, permission: &str) -> AuthResult<()> {
        let role = self.roles.get_mut(role_name)
            .ok_or_else(|| AuthError::RoleNotFound(role_name.to_string()))?;
        
        role.remove_permission(permission);
        self.role_cache.clear(); // Clear cache
        Ok(())
    }
    
    /// Assign role to user
    pub async fn assign_role(&mut self, user_id: &Uuid, role_name: &str) -> AuthResult<()> {
        if !self.roles.contains_key(role_name) {
            return Err(AuthError::RoleNotFound(role_name.to_string()));
        }
        
        let user_roles = self.user_roles.entry(*user_id).or_insert_with(HashSet::new);
        user_roles.insert(role_name.to_string());
        
        // Clear user's permission cache
        self.role_cache.remove(user_id);
        
        Ok(())
    }
    
    /// Remove role from user
    pub async fn remove_role(&mut self, user_id: &Uuid, role_name: &str) -> AuthResult<()> {
        if let Some(user_roles) = self.user_roles.get_mut(user_id) {
            user_roles.remove(role_name);
            
            // Clear user's permission cache
            self.role_cache.remove(user_id);
        }
        
        Ok(())
    }
    
    /// Get all roles for a user
    pub async fn get_user_roles(&self, user_id: &Uuid) -> Vec<String> {
        self.user_roles.get(user_id)
            .map(|roles| roles.iter().cloned().collect())
            .unwrap_or_default()
    }
    
    /// Get effective permissions for a user (including inherited permissions)
    pub async fn get_user_permissions(&mut self, user_id: &Uuid) -> HashSet<String> {
        // Check cache first
        if let Some(cached_permissions) = self.role_cache.get(user_id) {
            return cached_permissions.clone();
        }
        
        let mut permissions = HashSet::new();
        
        if let Some(user_roles) = self.user_roles.get(user_id) {
            for role_name in user_roles {
                self.collect_role_permissions(role_name, &mut permissions, &mut HashSet::new());
            }
        }
        
        // Cache the result
        self.role_cache.insert(*user_id, permissions.clone());
        
        permissions
    }
    
    /// Recursively collect permissions from role and its parent roles
    fn collect_role_permissions(&self, role_name: &str, permissions: &mut HashSet<String>, visited: &mut HashSet<String>) {
        if visited.contains(role_name) {
            return; // Prevent infinite recursion
        }
        
        visited.insert(role_name.to_string());
        
        if let Some(role) = self.roles.get(role_name) {
            // Add direct permissions
            for permission in &role.permissions {
                permissions.insert(permission.clone());
            }
            
            // Add permissions from parent roles
            for parent_role in &role.parent_roles {
                self.collect_role_permissions(parent_role, permissions, visited);
            }
        }
    }
    
    /// Check if user has a specific permission
    pub async fn check_permission(&mut self, user_id: &Uuid, permission: &str) -> AuthResult<bool> {
        let user_permissions = self.get_user_permissions(user_id).await;
        
        // Check direct permission match
        if user_permissions.contains(permission) {
            return Ok(true);
        }
        
        // Check wildcard permissions
        for user_perm in &user_permissions {
            if user_perm.ends_with(":*") {
                let prefix = &user_perm[..user_perm.len() - 1];
                if permission.starts_with(prefix) {
                    return Ok(true);
                }
            } else if user_perm == "admin:*" {
                return Ok(true); // Admin has all permissions
            }
        }
        
        Ok(false)
    }
    
    /// List all available roles
    pub fn list_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }
    
    /// List all available permissions
    pub fn list_permissions(&self) -> Vec<&Permission> {
        self.permissions.values().collect()
    }
    
    /// Get role by name
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }
    
    /// Get permission by name
    pub fn get_permission(&self, name: &str) -> Option<&Permission> {
        self.permissions.get(name)
    }
}

impl Default for RBACManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rbac_manager_creation() {
        let rbac = RBACManager::new();
        
        // Should have default roles
        assert!(rbac.get_role("user").is_some());
        assert!(rbac.get_role("developer").is_some());
        assert!(rbac.get_role("admin").is_some());
        
        // Should have default permissions
        assert!(rbac.get_permission("agents:create").is_some());
        assert!(rbac.get_permission("admin:*").is_some());
    }
    
    #[tokio::test]
    async fn test_role_assignment() {
        let mut rbac = RBACManager::new();
        let user_id = Uuid::new_v4();
        
        // Assign role
        rbac.assign_role(&user_id, "developer").await.unwrap();
        
        // Check role assignment
        let roles = rbac.get_user_roles(&user_id).await;
        assert!(roles.contains(&"developer".to_string()));
    }
    
    #[tokio::test]
    async fn test_permission_checking() {
        let mut rbac = RBACManager::new();
        let user_id = Uuid::new_v4();
        
        // Assign developer role
        rbac.assign_role(&user_id, "developer").await.unwrap();
        
        // Check permissions
        assert!(rbac.check_permission(&user_id, "agents:create").await.unwrap());
        assert!(rbac.check_permission(&user_id, "agents:read").await.unwrap());
        assert!(!rbac.check_permission(&user_id, "admin:delete").await.unwrap());
    }
    
    #[tokio::test]
    async fn test_role_inheritance() {
        let mut rbac = RBACManager::new();
        let user_id = Uuid::new_v4();
        
        // Assign developer role (which inherits from user role)
        rbac.assign_role(&user_id, "developer").await.unwrap();
        
        // Should have permissions from both developer and user roles
        let permissions = rbac.get_user_permissions(&user_id).await;
        assert!(permissions.contains("agents:create")); // from developer
        assert!(permissions.contains("agents:read")); // from both
    }
    
    #[tokio::test]
    async fn test_admin_wildcard_permissions() {
        let mut rbac = RBACManager::new();
        let user_id = Uuid::new_v4();
        
        // Assign admin role
        rbac.assign_role(&user_id, "admin").await.unwrap();
        
        // Admin should have all permissions
        assert!(rbac.check_permission(&user_id, "agents:create").await.unwrap());
        assert!(rbac.check_permission(&user_id, "tools:delete").await.unwrap());
        assert!(rbac.check_permission(&user_id, "any:permission").await.unwrap());
    }
}
