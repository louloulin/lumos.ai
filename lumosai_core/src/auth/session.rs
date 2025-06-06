//! Session Management System
//! 
//! This module provides secure session handling and lifecycle management
//! for web-based authentication in Lumos.ai.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::{AuthError, AuthResult, User};

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub expires_at: SystemTime,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Create a new session
    pub fn new(user_id: Uuid, tenant_id: Option<Uuid>, timeout: Duration) -> Self {
        let now = SystemTime::now();
        let session_id = Self::generate_session_id();
        
        Self {
            id: session_id,
            user_id,
            tenant_id,
            created_at: now,
            last_accessed: now,
            expires_at: now + timeout,
            ip_address: None,
            user_agent: None,
            is_active: true,
            metadata: HashMap::new(),
        }
    }
    
    /// Generate a secure session ID
    fn generate_session_id() -> String {
        format!("sess_{}", Uuid::new_v4().to_string().replace('-', ""))
    }
    
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
    
    /// Update last accessed time and extend expiration
    pub fn touch(&mut self, timeout: Duration) {
        let now = SystemTime::now();
        self.last_accessed = now;
        self.expires_at = now + timeout;
    }
    
    /// Invalidate the session
    pub fn invalidate(&mut self) {
        self.is_active = false;
    }
    
    /// Set client information
    pub fn set_client_info(&mut self, ip_address: Option<String>, user_agent: Option<String>) {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
    }
    
    /// Add metadata to session
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
    
    /// Get session duration
    pub fn duration(&self) -> Duration {
        self.last_accessed.duration_since(self.created_at).unwrap_or_default()
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: u64,
    pub active_sessions: u64,
    pub expired_sessions: u64,
    pub average_duration: Duration,
    pub peak_concurrent: u64,
}

/// Session Manager for session operations
#[derive(Debug)]
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    user_sessions: HashMap<Uuid, Vec<String>>, // user_id -> session_ids
    default_timeout: Duration,
    max_sessions_per_user: usize,
    stats: SessionStats,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(default_timeout: Duration) -> Self {
        Self {
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            default_timeout,
            max_sessions_per_user: 10, // Default limit
            stats: SessionStats {
                total_sessions: 0,
                active_sessions: 0,
                expired_sessions: 0,
                average_duration: Duration::from_secs(0),
                peak_concurrent: 0,
            },
        }
    }
    
    /// Create a new session for a user
    pub async fn create_session(&mut self, user_id: Uuid) -> AuthResult<String> {
        self.create_session_with_timeout(user_id, None, self.default_timeout).await
    }
    
    /// Create a new session with custom timeout
    pub async fn create_session_with_timeout(
        &mut self,
        user_id: Uuid,
        tenant_id: Option<Uuid>,
        timeout: Duration,
    ) -> AuthResult<String> {
        // Check session limit per user
        let oldest_session_to_remove = if let Some(user_session_list) = self.user_sessions.get(&user_id) {
            if user_session_list.len() >= self.max_sessions_per_user {
                // Get oldest session ID to remove
                user_session_list.first().cloned()
            } else {
                None
            }
        } else {
            None
        };

        // Remove oldest session if needed
        if let Some(oldest_session_id) = oldest_session_to_remove {
            self.invalidate_session(&oldest_session_id).await?;
        }
        
        let session = Session::new(user_id, tenant_id, timeout);
        let session_id = session.id.clone();
        
        // Store session
        self.sessions.insert(session_id.clone(), session);
        
        // Update user sessions index
        let user_session_list = self.user_sessions.entry(user_id).or_insert_with(Vec::new);
        user_session_list.push(session_id.clone());
        
        // Update stats
        self.stats.total_sessions += 1;
        self.stats.active_sessions += 1;
        self.stats.peak_concurrent = self.stats.peak_concurrent.max(self.stats.active_sessions);
        
        Ok(session_id)
    }
    
    /// Validate a session and return user information
    pub async fn validate_session(&mut self, session_id: &str) -> AuthResult<User> {
        let session = self.sessions.get_mut(session_id)
            .ok_or(AuthError::SessionExpired)?;
        
        // Check if session is active
        if !session.is_active {
            return Err(AuthError::SessionExpired);
        }
        
        // Check if session is expired
        if session.is_expired() {
            session.invalidate();
            self.stats.active_sessions = self.stats.active_sessions.saturating_sub(1);
            self.stats.expired_sessions += 1;
            return Err(AuthError::SessionExpired);
        }
        
        // Touch session to extend expiration
        session.touch(self.default_timeout);
        
        // Create user object from session
        // Note: In a real implementation, you would fetch full user data from database
        let user = User {
            id: session.user_id,
            email: format!("session-user-{}", session.user_id), // Placeholder
            username: None,
            tenant_id: session.tenant_id,
            roles: vec!["user".to_string()], // Default role
            permissions: vec!["basic:access".to_string()], // Default permission
            created_at: session.created_at,
            last_login: Some(session.last_accessed),
            is_active: true,
            metadata: session.metadata.clone(),
        };
        
        Ok(user)
    }
    
    /// Invalidate a session
    pub async fn invalidate_session(&mut self, session_id: &str) -> AuthResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            if session.is_active {
                session.invalidate();
                self.stats.active_sessions = self.stats.active_sessions.saturating_sub(1);

                // Remove from user sessions index
                if let Some(user_session_list) = self.user_sessions.get_mut(&session.user_id) {
                    user_session_list.retain(|id| id != session_id);
                }
            }
        }

        Ok(())
    }
    
    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(&mut self, user_id: &Uuid) -> AuthResult<usize> {
        let mut invalidated_count = 0;
        
        if let Some(session_ids) = self.user_sessions.get(user_id) {
            for session_id in session_ids {
                if let Some(session) = self.sessions.get_mut(session_id) {
                    if session.is_active {
                        session.invalidate();
                        self.stats.active_sessions = self.stats.active_sessions.saturating_sub(1);
                        invalidated_count += 1;
                    }
                }
            }
        }
        
        Ok(invalidated_count)
    }
    
    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }
    
    /// List all active sessions for a user
    pub async fn list_user_sessions(&self, user_id: &Uuid) -> Vec<&Session> {
        if let Some(session_ids) = self.user_sessions.get(user_id) {
            session_ids.iter()
                .filter_map(|session_id| self.sessions.get(session_id))
                .filter(|session| session.is_active && !session.is_expired())
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Update session metadata
    pub async fn update_session_metadata(
        &mut self,
        session_id: &str,
        metadata: HashMap<String, String>,
    ) -> AuthResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.metadata = metadata;
            Ok(())
        } else {
            Err(AuthError::SessionExpired)
        }
    }
    
    /// Set client information for session
    pub async fn set_session_client_info(
        &mut self,
        session_id: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AuthResult<()> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.set_client_info(ip_address, user_agent);
            Ok(())
        } else {
            Err(AuthError::SessionExpired)
        }
    }
    
    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&mut self) -> usize {
        let mut removed_count = 0;
        let mut sessions_to_remove = Vec::new();
        
        for (session_id, session) in &self.sessions {
            if session.is_expired() || !session.is_active {
                sessions_to_remove.push(session_id.clone());
            }
        }
        
        for session_id in sessions_to_remove {
            if let Some(session) = self.sessions.remove(&session_id) {
                // Remove from user sessions index
                if let Some(user_session_list) = self.user_sessions.get_mut(&session.user_id) {
                    user_session_list.retain(|id| id != &session_id);
                }
                
                if session.is_active {
                    self.stats.active_sessions = self.stats.active_sessions.saturating_sub(1);
                }
                
                self.stats.expired_sessions += 1;
                removed_count += 1;
            }
        }
        
        removed_count
    }
    
    /// Get session statistics
    pub fn get_stats(&self) -> &SessionStats {
        &self.stats
    }
    
    /// Set maximum sessions per user
    pub fn set_max_sessions_per_user(&mut self, max_sessions: usize) {
        self.max_sessions_per_user = max_sessions;
    }
    
    /// Get active session count
    pub fn active_session_count(&self) -> usize {
        self.sessions.values()
            .filter(|session| session.is_active && !session.is_expired())
            .count()
    }
    
    /// Get total session count
    pub fn total_session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_session_creation() {
        let mut manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = Uuid::new_v4();
        
        let session_id = manager.create_session(user_id).await.unwrap();
        
        assert!(session_id.starts_with("sess_"));
        assert!(!session_id.is_empty());
    }
    
    #[tokio::test]
    async fn test_session_validation() {
        let mut manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = Uuid::new_v4();
        
        let session_id = manager.create_session(user_id).await.unwrap();
        let user = manager.validate_session(&session_id).await.unwrap();
        
        assert_eq!(user.id, user_id);
    }
    
    #[tokio::test]
    async fn test_session_expiration() {
        let mut manager = SessionManager::new(Duration::from_secs(1));
        let user_id = Uuid::new_v4();
        
        let session_id = manager.create_session(user_id).await.unwrap();
        
        // Session should be valid initially
        assert!(manager.validate_session(&session_id).await.is_ok());
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Session should be expired now
        assert!(manager.validate_session(&session_id).await.is_err());
    }
    
    #[tokio::test]
    async fn test_session_invalidation() {
        let mut manager = SessionManager::new(Duration::from_secs(3600));
        let user_id = Uuid::new_v4();
        
        let session_id = manager.create_session(user_id).await.unwrap();
        
        // Session should be valid initially
        assert!(manager.validate_session(&session_id).await.is_ok());
        
        // Invalidate session
        manager.invalidate_session(&session_id).await.unwrap();
        
        // Session should be invalid now
        assert!(manager.validate_session(&session_id).await.is_err());
    }
    
    #[tokio::test]
    async fn test_user_session_limit() {
        let mut manager = SessionManager::new(Duration::from_secs(3600));
        manager.set_max_sessions_per_user(2);
        let user_id = Uuid::new_v4();
        
        // Create maximum allowed sessions
        let session1 = manager.create_session(user_id).await.unwrap();
        let session2 = manager.create_session(user_id).await.unwrap();
        
        // Both sessions should be valid
        assert!(manager.validate_session(&session1).await.is_ok());
        assert!(manager.validate_session(&session2).await.is_ok());
        
        // Create one more session (should invalidate the oldest)
        let session3 = manager.create_session(user_id).await.unwrap();
        
        // First session should be invalidated, others should be valid
        assert!(manager.validate_session(&session1).await.is_err());
        assert!(manager.validate_session(&session2).await.is_ok());
        assert!(manager.validate_session(&session3).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_sessions() {
        let mut manager = SessionManager::new(Duration::from_secs(1));
        let user_id = Uuid::new_v4();
        
        // Create a session that will expire
        manager.create_session(user_id).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Clean up expired sessions
        let removed_count = manager.cleanup_expired_sessions().await;
        assert_eq!(removed_count, 1);
        
        // Total session count should be 0 now
        assert_eq!(manager.total_session_count(), 0);
    }
}
