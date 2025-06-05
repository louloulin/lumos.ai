//! Multi-Tenant Support System
//! 
//! This module provides comprehensive multi-tenant architecture support
//! for enterprise deployments of Lumos.ai.

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::{AuthError, AuthResult};

/// Tenant subscription plan
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionPlan {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom,
}

impl SubscriptionPlan {
    /// Get resource limits for the plan
    pub fn get_limits(&self) -> ResourceLimits {
        match self {
            SubscriptionPlan::Free => ResourceLimits {
                max_users: 5,
                max_agents: 10,
                max_tools: 20,
                max_workflows: 5,
                max_api_calls_per_month: 1000,
                max_storage_gb: 1,
                max_concurrent_executions: 2,
                custom_integrations: false,
                priority_support: false,
            },
            SubscriptionPlan::Starter => ResourceLimits {
                max_users: 25,
                max_agents: 50,
                max_tools: 100,
                max_workflows: 25,
                max_api_calls_per_month: 10000,
                max_storage_gb: 10,
                max_concurrent_executions: 5,
                custom_integrations: false,
                priority_support: false,
            },
            SubscriptionPlan::Professional => ResourceLimits {
                max_users: 100,
                max_agents: 200,
                max_tools: 500,
                max_workflows: 100,
                max_api_calls_per_month: 100000,
                max_storage_gb: 100,
                max_concurrent_executions: 20,
                custom_integrations: true,
                priority_support: true,
            },
            SubscriptionPlan::Enterprise => ResourceLimits {
                max_users: 1000,
                max_agents: 1000,
                max_tools: 2000,
                max_workflows: 500,
                max_api_calls_per_month: 1000000,
                max_storage_gb: 1000,
                max_concurrent_executions: 100,
                custom_integrations: true,
                priority_support: true,
            },
            SubscriptionPlan::Custom => ResourceLimits {
                max_users: u32::MAX,
                max_agents: u32::MAX,
                max_tools: u32::MAX,
                max_workflows: u32::MAX,
                max_api_calls_per_month: u64::MAX,
                max_storage_gb: u64::MAX,
                max_concurrent_executions: u32::MAX,
                custom_integrations: true,
                priority_support: true,
            },
        }
    }
}

/// Resource limits for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_users: u32,
    pub max_agents: u32,
    pub max_tools: u32,
    pub max_workflows: u32,
    pub max_api_calls_per_month: u64,
    pub max_storage_gb: u64,
    pub max_concurrent_executions: u32,
    pub custom_integrations: bool,
    pub priority_support: bool,
}

/// Current resource usage for a tenant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub current_users: u32,
    pub current_agents: u32,
    pub current_tools: u32,
    pub current_workflows: u32,
    pub api_calls_this_month: u64,
    pub storage_used_gb: u64,
    pub concurrent_executions: u32,
    pub last_updated: SystemTime,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            current_users: 0,
            current_agents: 0,
            current_tools: 0,
            current_workflows: 0,
            api_calls_this_month: 0,
            storage_used_gb: 0,
            concurrent_executions: 0,
            last_updated: SystemTime::now(),
        }
    }
}

/// Tenant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub name: String,
    pub domain: Option<String>,
    pub subscription_plan: SubscriptionPlan,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
    pub is_active: bool,
    pub owner_user_id: Uuid,
    pub settings: TenantSettings,
    pub metadata: HashMap<String, String>,
}

impl Tenant {
    /// Create a new tenant
    pub fn new(name: String, owner_user_id: Uuid, subscription_plan: SubscriptionPlan) -> Self {
        let now = SystemTime::now();
        
        Self {
            id: Uuid::new_v4(),
            name,
            domain: None,
            subscription_plan,
            created_at: now,
            updated_at: now,
            is_active: true,
            owner_user_id,
            settings: TenantSettings::default(),
            metadata: HashMap::new(),
        }
    }
    
    /// Get resource limits for this tenant
    pub fn get_limits(&self) -> ResourceLimits {
        self.subscription_plan.get_limits()
    }
    
    /// Update subscription plan
    pub fn update_subscription(&mut self, plan: SubscriptionPlan) {
        self.subscription_plan = plan;
        self.updated_at = SystemTime::now();
    }
    
    /// Deactivate tenant
    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = SystemTime::now();
    }
    
    /// Set custom domain
    pub fn set_domain(&mut self, domain: String) {
        self.domain = Some(domain);
        self.updated_at = SystemTime::now();
    }
}

/// Tenant-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    pub allow_user_registration: bool,
    pub require_email_verification: bool,
    pub enable_sso: bool,
    pub enable_audit_logs: bool,
    pub data_retention_days: u32,
    pub allowed_ip_ranges: Vec<String>,
    pub custom_branding: bool,
    pub api_rate_limit_per_minute: u32,
}

impl Default for TenantSettings {
    fn default() -> Self {
        Self {
            allow_user_registration: true,
            require_email_verification: false,
            enable_sso: false,
            enable_audit_logs: true,
            data_retention_days: 365,
            allowed_ip_ranges: Vec::new(),
            custom_branding: false,
            api_rate_limit_per_minute: 100,
        }
    }
}

/// Multi-Tenant Manager
#[derive(Debug)]
pub struct TenantManager {
    tenants: HashMap<Uuid, Tenant>,
    domain_mapping: HashMap<String, Uuid>, // domain -> tenant_id
    usage_tracking: HashMap<Uuid, ResourceUsage>, // tenant_id -> usage
}

impl TenantManager {
    /// Create a new tenant manager
    pub fn new() -> Self {
        Self {
            tenants: HashMap::new(),
            domain_mapping: HashMap::new(),
            usage_tracking: HashMap::new(),
        }
    }
    
    /// Create a new tenant
    pub async fn create_tenant(
        &mut self,
        name: String,
        owner_user_id: Uuid,
        subscription_plan: SubscriptionPlan,
    ) -> AuthResult<Tenant> {
        let tenant = Tenant::new(name, owner_user_id, subscription_plan);
        let tenant_id = tenant.id;
        
        // Store tenant
        self.tenants.insert(tenant_id, tenant.clone());
        
        // Initialize usage tracking
        self.usage_tracking.insert(tenant_id, ResourceUsage::default());
        
        Ok(tenant)
    }
    
    /// Get tenant by ID
    pub async fn get_tenant(&self, tenant_id: &Uuid) -> Option<&Tenant> {
        self.tenants.get(tenant_id)
    }
    
    /// Get tenant by domain
    pub async fn get_tenant_by_domain(&self, domain: &str) -> Option<&Tenant> {
        if let Some(tenant_id) = self.domain_mapping.get(domain) {
            self.tenants.get(tenant_id)
        } else {
            None
        }
    }
    
    /// Update tenant
    pub async fn update_tenant(&mut self, tenant_id: &Uuid, tenant: Tenant) -> AuthResult<()> {
        if let Some(existing_tenant) = self.tenants.get_mut(tenant_id) {
            // Update domain mapping if domain changed
            if existing_tenant.domain != tenant.domain {
                // Remove old domain mapping
                if let Some(old_domain) = &existing_tenant.domain {
                    self.domain_mapping.remove(old_domain);
                }
                
                // Add new domain mapping
                if let Some(new_domain) = &tenant.domain {
                    self.domain_mapping.insert(new_domain.clone(), *tenant_id);
                }
            }
            
            *existing_tenant = tenant;
            Ok(())
        } else {
            Err(AuthError::TenantNotFound(tenant_id.to_string()))
        }
    }
    
    /// Delete tenant
    pub async fn delete_tenant(&mut self, tenant_id: &Uuid) -> AuthResult<()> {
        if let Some(tenant) = self.tenants.remove(tenant_id) {
            // Remove domain mapping
            if let Some(domain) = &tenant.domain {
                self.domain_mapping.remove(domain);
            }
            
            // Remove usage tracking
            self.usage_tracking.remove(tenant_id);
            
            Ok(())
        } else {
            Err(AuthError::TenantNotFound(tenant_id.to_string()))
        }
    }
    
    /// List all tenants
    pub async fn list_tenants(&self) -> Vec<&Tenant> {
        self.tenants.values().collect()
    }
    
    /// Check if tenant can perform an action based on resource limits
    pub async fn check_resource_limit(
        &self,
        tenant_id: &Uuid,
        resource_type: &str,
        requested_amount: u32,
    ) -> AuthResult<bool> {
        let tenant = self.tenants.get(tenant_id)
            .ok_or_else(|| AuthError::TenantNotFound(tenant_id.to_string()))?;
        
        let limits = tenant.get_limits();
        let default_usage = ResourceUsage::default();
        let usage = self.usage_tracking.get(tenant_id)
            .unwrap_or(&default_usage);
        
        let can_proceed = match resource_type {
            "users" => usage.current_users + requested_amount <= limits.max_users,
            "agents" => usage.current_agents + requested_amount <= limits.max_agents,
            "tools" => usage.current_tools + requested_amount <= limits.max_tools,
            "workflows" => usage.current_workflows + requested_amount <= limits.max_workflows,
            "concurrent_executions" => usage.concurrent_executions + requested_amount <= limits.max_concurrent_executions,
            _ => true, // Unknown resource type, allow by default
        };
        
        Ok(can_proceed)
    }
    
    /// Update resource usage for a tenant
    pub async fn update_usage(
        &mut self,
        tenant_id: &Uuid,
        resource_type: &str,
        delta: i32,
    ) -> AuthResult<()> {
        let usage = self.usage_tracking.entry(*tenant_id).or_insert_with(ResourceUsage::default);
        
        match resource_type {
            "users" => {
                usage.current_users = (usage.current_users as i32 + delta).max(0) as u32;
            },
            "agents" => {
                usage.current_agents = (usage.current_agents as i32 + delta).max(0) as u32;
            },
            "tools" => {
                usage.current_tools = (usage.current_tools as i32 + delta).max(0) as u32;
            },
            "workflows" => {
                usage.current_workflows = (usage.current_workflows as i32 + delta).max(0) as u32;
            },
            "api_calls" => {
                usage.api_calls_this_month = (usage.api_calls_this_month as i64 + delta as i64).max(0) as u64;
            },
            "concurrent_executions" => {
                usage.concurrent_executions = (usage.concurrent_executions as i32 + delta).max(0) as u32;
            },
            _ => {}, // Unknown resource type, ignore
        }
        
        usage.last_updated = SystemTime::now();
        Ok(())
    }
    
    /// Get resource usage for a tenant
    pub async fn get_usage(&self, tenant_id: &Uuid) -> Option<&ResourceUsage> {
        self.usage_tracking.get(tenant_id)
    }
    
    /// Set custom domain for tenant
    pub async fn set_tenant_domain(&mut self, tenant_id: &Uuid, domain: String) -> AuthResult<()> {
        // Check if domain is already taken
        if self.domain_mapping.contains_key(&domain) {
            return Err(AuthError::Other("Domain already in use".to_string()));
        }
        
        if let Some(tenant) = self.tenants.get_mut(tenant_id) {
            // Remove old domain mapping
            if let Some(old_domain) = &tenant.domain {
                self.domain_mapping.remove(old_domain);
            }
            
            // Set new domain
            tenant.set_domain(domain.clone());
            self.domain_mapping.insert(domain, *tenant_id);
            
            Ok(())
        } else {
            Err(AuthError::TenantNotFound(tenant_id.to_string()))
        }
    }
    
    /// Update tenant settings
    pub async fn update_tenant_settings(
        &mut self,
        tenant_id: &Uuid,
        settings: TenantSettings,
    ) -> AuthResult<()> {
        if let Some(tenant) = self.tenants.get_mut(tenant_id) {
            tenant.settings = settings;
            tenant.updated_at = SystemTime::now();
            Ok(())
        } else {
            Err(AuthError::TenantNotFound(tenant_id.to_string()))
        }
    }
    
    /// Check if IP address is allowed for tenant
    pub async fn is_ip_allowed(&self, tenant_id: &Uuid, ip_address: &str) -> AuthResult<bool> {
        let tenant = self.tenants.get(tenant_id)
            .ok_or_else(|| AuthError::TenantNotFound(tenant_id.to_string()))?;
        
        // If no IP restrictions are set, allow all
        if tenant.settings.allowed_ip_ranges.is_empty() {
            return Ok(true);
        }
        
        // Check if IP is in allowed ranges
        // This is a simplified implementation - in production, use proper CIDR matching
        for allowed_range in &tenant.settings.allowed_ip_ranges {
            if ip_address.starts_with(allowed_range) {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}

impl Default for TenantManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tenant_creation() {
        let mut manager = TenantManager::new();
        let owner_id = Uuid::new_v4();
        
        let tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            owner_id,
            SubscriptionPlan::Professional,
        ).await.unwrap();
        
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.owner_user_id, owner_id);
        assert_eq!(tenant.subscription_plan, SubscriptionPlan::Professional);
    }
    
    #[tokio::test]
    async fn test_resource_limits() {
        let free_limits = SubscriptionPlan::Free.get_limits();
        let enterprise_limits = SubscriptionPlan::Enterprise.get_limits();
        
        assert!(free_limits.max_users < enterprise_limits.max_users);
        assert!(free_limits.max_api_calls_per_month < enterprise_limits.max_api_calls_per_month);
        assert!(!free_limits.custom_integrations);
        assert!(enterprise_limits.custom_integrations);
    }
    
    #[tokio::test]
    async fn test_resource_limit_checking() {
        let mut manager = TenantManager::new();
        let owner_id = Uuid::new_v4();
        
        let tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            owner_id,
            SubscriptionPlan::Free,
        ).await.unwrap();
        
        // Should be able to add users within limit
        let can_add_users = manager.check_resource_limit(&tenant.id, "users", 3).await.unwrap();
        assert!(can_add_users);
        
        // Should not be able to add users beyond limit
        let cannot_add_users = manager.check_resource_limit(&tenant.id, "users", 10).await.unwrap();
        assert!(!cannot_add_users);
    }
    
    #[tokio::test]
    async fn test_domain_mapping() {
        let mut manager = TenantManager::new();
        let owner_id = Uuid::new_v4();
        
        let tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            owner_id,
            SubscriptionPlan::Professional,
        ).await.unwrap();
        
        // Set domain
        manager.set_tenant_domain(&tenant.id, "example.com".to_string()).await.unwrap();
        
        // Should be able to find tenant by domain
        let found_tenant = manager.get_tenant_by_domain("example.com").await;
        assert!(found_tenant.is_some());
        assert_eq!(found_tenant.unwrap().id, tenant.id);
    }
    
    #[tokio::test]
    async fn test_usage_tracking() {
        let mut manager = TenantManager::new();
        let owner_id = Uuid::new_v4();
        
        let tenant = manager.create_tenant(
            "Test Tenant".to_string(),
            owner_id,
            SubscriptionPlan::Professional,
        ).await.unwrap();
        
        // Update usage
        manager.update_usage(&tenant.id, "users", 5).await.unwrap();
        manager.update_usage(&tenant.id, "agents", 10).await.unwrap();
        
        // Check usage
        let usage = manager.get_usage(&tenant.id).await.unwrap();
        assert_eq!(usage.current_users, 5);
        assert_eq!(usage.current_agents, 10);
    }
}
