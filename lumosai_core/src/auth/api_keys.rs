//! API Key Management System
//! 
//! This module provides secure API key generation, validation, and management
//! for programmatic access to Lumos.ai services.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::{AuthError, AuthResult, User};

/// API Key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
    pub last_used: Option<SystemTime>,
    pub is_active: bool,
    pub usage_count: u64,
    pub rate_limit: Option<u32>, // requests per minute
    pub metadata: HashMap<String, String>,
}

impl ApiKey {
    /// Create a new API key
    pub fn new(
        name: String,
        key_hash: String,
        user_id: Uuid,
        tenant_id: Option<Uuid>,
        scopes: Vec<String>,
        expires_at: Option<SystemTime>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            key_hash,
            user_id,
            tenant_id,
            scopes,
            created_at: SystemTime::now(),
            expires_at,
            last_used: None,
            is_active: true,
            usage_count: 0,
            rate_limit: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if API key is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            SystemTime::now() > expires_at
        } else {
            false
        }
    }
    
    /// Check if API key has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string()) || self.scopes.contains(&"*".to_string())
    }
    
    /// Update last used timestamp and increment usage count
    pub fn mark_used(&mut self) {
        self.last_used = Some(SystemTime::now());
        self.usage_count += 1;
    }
    
    /// Deactivate the API key
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
    
    /// Set rate limit
    pub fn set_rate_limit(&mut self, requests_per_minute: u32) {
        self.rate_limit = Some(requests_per_minute);
    }
}

/// API Key usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyUsage {
    pub key_id: Uuid,
    pub total_requests: u64,
    pub requests_today: u64,
    pub requests_this_month: u64,
    pub last_request: Option<SystemTime>,
    pub error_count: u64,
    pub rate_limit_hits: u64,
}

/// API Key Manager for key operations
#[derive(Debug)]
pub struct ApiKeyManager {
    keys: HashMap<String, ApiKey>, // key_hash -> ApiKey
    user_keys: HashMap<Uuid, Vec<Uuid>>, // user_id -> key_ids
    usage_stats: HashMap<Uuid, ApiKeyUsage>, // key_id -> usage
    default_expiration: Duration,
}

impl ApiKeyManager {
    /// Create a new API key manager
    pub fn new(default_expiration: Duration) -> Self {
        Self {
            keys: HashMap::new(),
            user_keys: HashMap::new(),
            usage_stats: HashMap::new(),
            default_expiration,
        }
    }
    
    /// Generate a new API key for a user
    pub async fn generate_key(
        &mut self,
        user_id: Uuid,
        name: &str,
        scopes: Vec<String>,
    ) -> AuthResult<String> {
        self.generate_key_with_expiration(user_id, name, scopes, None).await
    }
    
    /// Generate a new API key with custom expiration
    pub async fn generate_key_with_expiration(
        &mut self,
        user_id: Uuid,
        name: &str,
        scopes: Vec<String>,
        expires_at: Option<SystemTime>,
    ) -> AuthResult<String> {
        // Generate a secure random key
        let key = self.generate_secure_key();
        let key_hash = self.hash_key(&key);
        
        let expiration = expires_at.unwrap_or_else(|| {
            SystemTime::now() + self.default_expiration
        });
        
        let api_key = ApiKey::new(
            name.to_string(),
            key_hash.clone(),
            user_id,
            None, // TODO: Get tenant_id from user context
            scopes,
            Some(expiration),
        );
        
        // Store the key
        self.keys.insert(key_hash, api_key.clone());
        
        // Update user keys index
        let user_key_list = self.user_keys.entry(user_id).or_insert_with(Vec::new);
        user_key_list.push(api_key.id);
        
        // Initialize usage stats
        self.usage_stats.insert(api_key.id, ApiKeyUsage {
            key_id: api_key.id,
            total_requests: 0,
            requests_today: 0,
            requests_this_month: 0,
            last_request: None,
            error_count: 0,
            rate_limit_hits: 0,
        });
        
        Ok(key)
    }
    
    /// Validate an API key and return user information
    pub async fn validate_key(&mut self, key: &str) -> AuthResult<User> {
        let key_hash = self.hash_key(key);
        
        let api_key = self.keys.get_mut(&key_hash)
            .ok_or(AuthError::InvalidApiKey)?;
        
        // Check if key is active
        if !api_key.is_active {
            return Err(AuthError::InvalidApiKey);
        }
        
        // Check if key is expired
        if api_key.is_expired() {
            return Err(AuthError::InvalidApiKey);
        }
        
        // Update usage
        api_key.mark_used();
        
        // Update usage stats
        if let Some(stats) = self.usage_stats.get_mut(&api_key.id) {
            stats.total_requests += 1;
            stats.last_request = Some(SystemTime::now());
            // TODO: Update daily/monthly counters based on date
        }
        
        // Create user object from API key
        let user = User {
            id: api_key.user_id,
            email: format!("api-key-{}", api_key.name), // Placeholder email
            username: Some(format!("api-{}", api_key.name)),
            tenant_id: api_key.tenant_id,
            roles: api_key.scopes.clone(), // Use scopes as roles for API keys
            permissions: api_key.scopes.clone(),
            created_at: api_key.created_at,
            last_login: api_key.last_used,
            is_active: api_key.is_active,
            metadata: api_key.metadata.clone(),
        };
        
        Ok(user)
    }
    
    /// Revoke an API key
    pub async fn revoke_key(&mut self, key: &str) -> AuthResult<()> {
        let key_hash = self.hash_key(key);
        
        if let Some(api_key) = self.keys.get_mut(&key_hash) {
            api_key.deactivate();
            Ok(())
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }
    
    /// Revoke API key by ID
    pub async fn revoke_key_by_id(&mut self, key_id: Uuid) -> AuthResult<()> {
        for api_key in self.keys.values_mut() {
            if api_key.id == key_id {
                api_key.deactivate();
                return Ok(());
            }
        }
        
        Err(AuthError::Other("API key not found".to_string()))
    }
    
    /// List all API keys for a user
    pub async fn list_user_keys(&self, user_id: &Uuid) -> Vec<&ApiKey> {
        if let Some(key_ids) = self.user_keys.get(user_id) {
            key_ids.iter()
                .filter_map(|key_id| {
                    self.keys.values().find(|key| key.id == *key_id)
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get API key by ID
    pub async fn get_key_by_id(&self, key_id: &Uuid) -> Option<&ApiKey> {
        self.keys.values().find(|key| key.id == *key_id)
    }
    
    /// Get usage statistics for an API key
    pub async fn get_key_usage(&self, key_id: &Uuid) -> Option<&ApiKeyUsage> {
        self.usage_stats.get(key_id)
    }
    
    /// Update API key metadata
    pub async fn update_key_metadata(
        &mut self,
        key_id: Uuid,
        metadata: HashMap<String, String>,
    ) -> AuthResult<()> {
        for api_key in self.keys.values_mut() {
            if api_key.id == key_id {
                api_key.metadata = metadata;
                return Ok(());
            }
        }
        
        Err(AuthError::Other("API key not found".to_string()))
    }
    
    /// Set rate limit for an API key
    pub async fn set_rate_limit(&mut self, key_id: Uuid, requests_per_minute: u32) -> AuthResult<()> {
        for api_key in self.keys.values_mut() {
            if api_key.id == key_id {
                api_key.set_rate_limit(requests_per_minute);
                return Ok(());
            }
        }
        
        Err(AuthError::Other("API key not found".to_string()))
    }
    
    /// Check if API key has exceeded rate limit
    pub async fn check_rate_limit(&self, key: &str) -> AuthResult<bool> {
        let key_hash = self.hash_key(key);
        
        if let Some(api_key) = self.keys.get(&key_hash) {
            if let Some(rate_limit) = api_key.rate_limit {
                // TODO: Implement actual rate limiting logic
                // This would typically involve checking request timestamps
                // within the last minute and comparing to the limit
                Ok(false) // For now, never rate limited
            } else {
                Ok(false) // No rate limit set
            }
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }
    
    /// Generate a secure random API key
    fn generate_secure_key(&self) -> String {
        // In production, use a cryptographically secure random generator
        // For now, use a simple UUID-based approach
        let prefix = "lum"; // Lumos prefix
        let key_part = Uuid::new_v4().to_string().replace('-', "");
        format!("{}_{}", prefix, key_part)
    }
    
    /// Hash an API key for secure storage
    fn hash_key(&self, key: &str) -> String {
        // In production, use a proper cryptographic hash function like SHA-256
        // For now, use a simple hash for demonstration
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// Clean up expired keys
    pub async fn cleanup_expired_keys(&mut self) -> usize {
        let mut removed_count = 0;
        let mut keys_to_remove = Vec::new();
        
        for (key_hash, api_key) in &self.keys {
            if api_key.is_expired() {
                keys_to_remove.push(key_hash.clone());
            }
        }
        
        for key_hash in keys_to_remove {
            if let Some(api_key) = self.keys.remove(&key_hash) {
                // Remove from user keys index
                if let Some(user_key_list) = self.user_keys.get_mut(&api_key.user_id) {
                    user_key_list.retain(|&id| id != api_key.id);
                }
                
                // Remove usage stats
                self.usage_stats.remove(&api_key.id);
                
                removed_count += 1;
            }
        }
        
        removed_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_api_key_generation() {
        let mut manager = ApiKeyManager::new(Duration::from_secs(3600));
        let user_id = Uuid::new_v4();
        
        let key = manager.generate_key(user_id, "test-key", vec!["agents:read".to_string()]).await.unwrap();
        
        assert!(key.starts_with("lum_"));
        assert!(!key.is_empty());
    }
    
    #[tokio::test]
    async fn test_api_key_validation() {
        let mut manager = ApiKeyManager::new(Duration::from_secs(3600));
        let user_id = Uuid::new_v4();
        
        let key = manager.generate_key(user_id, "test-key", vec!["agents:read".to_string()]).await.unwrap();
        let user = manager.validate_key(&key).await.unwrap();
        
        assert_eq!(user.id, user_id);
        assert!(user.has_permission("agents:read"));
    }
    
    #[tokio::test]
    async fn test_api_key_revocation() {
        let mut manager = ApiKeyManager::new(Duration::from_secs(3600));
        let user_id = Uuid::new_v4();
        
        let key = manager.generate_key(user_id, "test-key", vec!["agents:read".to_string()]).await.unwrap();
        
        // Key should be valid initially
        assert!(manager.validate_key(&key).await.is_ok());
        
        // Revoke the key
        manager.revoke_key(&key).await.unwrap();
        
        // Key should be invalid now
        assert!(manager.validate_key(&key).await.is_err());
    }
    
    #[tokio::test]
    async fn test_api_key_expiration() {
        let mut manager = ApiKeyManager::new(Duration::from_secs(1));
        let user_id = Uuid::new_v4();
        
        let key = manager.generate_key(user_id, "test-key", vec!["agents:read".to_string()]).await.unwrap();
        
        // Key should be valid initially
        assert!(manager.validate_key(&key).await.is_ok());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Key should be expired now
        assert!(manager.validate_key(&key).await.is_err());
    }
    
    #[tokio::test]
    async fn test_list_user_keys() {
        let mut manager = ApiKeyManager::new(Duration::from_secs(3600));
        let user_id = Uuid::new_v4();
        
        // Generate multiple keys
        manager.generate_key(user_id, "key1", vec!["agents:read".to_string()]).await.unwrap();
        manager.generate_key(user_id, "key2", vec!["tools:read".to_string()]).await.unwrap();
        
        let keys = manager.list_user_keys(&user_id).await;
        assert_eq!(keys.len(), 2);
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_keys() {
        let mut manager = ApiKeyManager::new(Duration::from_secs(1));
        let user_id = Uuid::new_v4();
        
        // Generate a key that will expire
        manager.generate_key(user_id, "test-key", vec!["agents:read".to_string()]).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Clean up expired keys
        let removed_count = manager.cleanup_expired_keys().await;
        assert_eq!(removed_count, 1);
        
        // User should have no keys now
        let keys = manager.list_user_keys(&user_id).await;
        assert_eq!(keys.len(), 0);
    }
}
