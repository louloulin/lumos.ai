//! OAuth2 Authentication Support
//! 
//! This module provides OAuth2 integration for external identity providers
//! such as Google, GitHub, Microsoft, etc.

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::{AuthError, AuthResult, AuthToken, TokenType, User};

/// OAuth2 Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Provider {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
    pub enabled: bool,
}

impl OAuth2Provider {
    /// Create a new OAuth2 provider configuration
    pub fn new(
        name: String,
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        user_info_url: String,
    ) -> Self {
        Self {
            name,
            client_id,
            client_secret,
            auth_url,
            token_url,
            user_info_url,
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            redirect_uri: "http://localhost:3000/auth/callback".to_string(),
            enabled: true,
        }
    }
    
    /// Create Google OAuth2 provider
    pub fn google(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            name: "google".to_string(),
            client_id,
            client_secret,
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            user_info_url: "https://www.googleapis.com/oauth2/v2/userinfo".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string(), "email".to_string()],
            redirect_uri,
            enabled: true,
        }
    }
    
    /// Create GitHub OAuth2 provider
    pub fn github(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            name: "github".to_string(),
            client_id,
            client_secret,
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            user_info_url: "https://api.github.com/user".to_string(),
            scopes: vec!["user:email".to_string()],
            redirect_uri,
            enabled: true,
        }
    }
    
    /// Generate authorization URL
    pub fn get_auth_url(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&scope={}&response_type=code&state={}",
            self.auth_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state)
        )
    }
}

/// OAuth2 authorization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2State {
    pub state: String,
    pub provider: String,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub redirect_url: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl OAuth2State {
    /// Create a new OAuth2 state
    pub fn new(provider: String, redirect_url: Option<String>) -> Self {
        let now = SystemTime::now();
        let state = Uuid::new_v4().to_string();
        
        Self {
            state,
            provider,
            created_at: now,
            expires_at: now + Duration::from_secs(600), // 10 minutes
            redirect_url,
            metadata: HashMap::new(),
        }
    }
    
    /// Check if state is expired
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }
}

/// OAuth2 token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub id_token: Option<String>,
}

/// OAuth2 user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2UserInfo {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub verified_email: Option<bool>,
    pub provider: String,
    pub raw_data: HashMap<String, serde_json::Value>,
}

/// OAuth2 Manager for handling OAuth2 flows
#[derive(Debug)]
pub struct OAuth2Manager {
    providers: HashMap<String, OAuth2Provider>,
    states: HashMap<String, OAuth2State>,
}

impl OAuth2Manager {
    /// Create a new OAuth2 manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            states: HashMap::new(),
        }
    }
    
    /// Add an OAuth2 provider
    pub fn add_provider(&mut self, provider: OAuth2Provider) {
        self.providers.insert(provider.name.clone(), provider);
    }
    
    /// Remove an OAuth2 provider
    pub fn remove_provider(&mut self, name: &str) {
        self.providers.remove(name);
    }
    
    /// Get OAuth2 provider by name
    pub fn get_provider(&self, name: &str) -> Option<&OAuth2Provider> {
        self.providers.get(name)
    }
    
    /// List all available providers
    pub fn list_providers(&self) -> Vec<&OAuth2Provider> {
        self.providers.values().filter(|p| p.enabled).collect()
    }
    
    /// Start OAuth2 authorization flow
    pub async fn start_authorization(
        &mut self,
        provider_name: &str,
        redirect_url: Option<String>,
    ) -> AuthResult<String> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| AuthError::Other(format!("OAuth2 provider '{}' not found", provider_name)))?;
        
        if !provider.enabled {
            return Err(AuthError::Other(format!("OAuth2 provider '{}' is disabled", provider_name)));
        }
        
        let state = OAuth2State::new(provider_name.to_string(), redirect_url);
        let state_param = state.state.clone();
        
        // Store state for validation
        self.states.insert(state_param.clone(), state);
        
        // Generate authorization URL
        let auth_url = provider.get_auth_url(&state_param);
        
        Ok(auth_url)
    }
    
    /// Handle OAuth2 callback and exchange code for token
    pub async fn handle_callback(
        &mut self,
        code: &str,
        state: &str,
    ) -> AuthResult<AuthToken> {
        // Validate state
        let oauth_state = self.states.remove(state)
            .ok_or_else(|| AuthError::InvalidToken("Invalid OAuth2 state".to_string()))?;
        
        if oauth_state.is_expired() {
            return Err(AuthError::TokenExpired);
        }
        
        let provider = self.providers.get(&oauth_state.provider)
            .ok_or_else(|| AuthError::Other("OAuth2 provider not found".to_string()))?;
        
        // Exchange code for token (mock implementation)
        let token_response = self.exchange_code_for_token(provider, code).await?;
        
        // Get user information
        let user_info = self.get_user_info(provider, &token_response.access_token).await?;
        
        // Create user from OAuth2 info
        let user = self.create_user_from_oauth2(&user_info)?;
        
        // Create auth token
        let auth_token = AuthToken {
            token: token_response.access_token,
            token_type: TokenType::OAuth2,
            user_id: user.id,
            tenant_id: user.tenant_id,
            expires_at: SystemTime::now() + Duration::from_secs(
                token_response.expires_in.unwrap_or(3600)
            ),
            scopes: token_response.scope
                .unwrap_or_default()
                .split(' ')
                .map(|s| s.to_string())
                .collect(),
            metadata: HashMap::new(),
        };
        
        Ok(auth_token)
    }
    
    /// Exchange authorization code for access token (mock implementation)
    async fn exchange_code_for_token(
        &self,
        provider: &OAuth2Provider,
        code: &str,
    ) -> AuthResult<OAuth2TokenResponse> {
        // In a real implementation, this would make an HTTP POST request to the token endpoint
        // For now, return a mock response
        Ok(OAuth2TokenResponse {
            access_token: format!("oauth2_token_{}", Uuid::new_v4()),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some(format!("refresh_token_{}", Uuid::new_v4())),
            scope: Some(provider.scopes.join(" ")),
            id_token: None,
        })
    }
    
    /// Get user information from OAuth2 provider (mock implementation)
    async fn get_user_info(
        &self,
        provider: &OAuth2Provider,
        access_token: &str,
    ) -> AuthResult<OAuth2UserInfo> {
        // In a real implementation, this would make an HTTP GET request to the user info endpoint
        // For now, return a mock response
        Ok(OAuth2UserInfo {
            id: format!("oauth2_user_{}", Uuid::new_v4()),
            email: Some("user@example.com".to_string()),
            name: Some("OAuth2 User".to_string()),
            picture: None,
            verified_email: Some(true),
            provider: provider.name.clone(),
            raw_data: HashMap::new(),
        })
    }
    
    /// Create user from OAuth2 user information
    fn create_user_from_oauth2(&self, user_info: &OAuth2UserInfo) -> AuthResult<User> {
        let email = user_info.email.clone()
            .ok_or_else(|| AuthError::Other("Email not provided by OAuth2 provider".to_string()))?;
        
        let mut user = User::new(email, None);
        user.username = user_info.name.clone();
        
        // Add OAuth2 metadata
        user.metadata.insert("oauth2_provider".to_string(), user_info.provider.clone());
        user.metadata.insert("oauth2_id".to_string(), user_info.id.clone());
        
        if let Some(picture) = &user_info.picture {
            user.metadata.insert("picture".to_string(), picture.clone());
        }
        
        // Assign default role for OAuth2 users
        user.roles.push("user".to_string());
        user.permissions.push("basic:access".to_string());
        
        Ok(user)
    }
    
    /// Refresh OAuth2 token
    pub async fn refresh_token(
        &self,
        provider_name: &str,
        refresh_token: &str,
    ) -> AuthResult<OAuth2TokenResponse> {
        let provider = self.providers.get(provider_name)
            .ok_or_else(|| AuthError::Other(format!("OAuth2 provider '{}' not found", provider_name)))?;
        
        // In a real implementation, this would make an HTTP POST request to refresh the token
        // For now, return a mock response
        Ok(OAuth2TokenResponse {
            access_token: format!("refreshed_token_{}", Uuid::new_v4()),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some(refresh_token.to_string()),
            scope: Some(provider.scopes.join(" ")),
            id_token: None,
        })
    }
    
    /// Clean up expired states
    pub async fn cleanup_expired_states(&mut self) -> usize {
        let mut removed_count = 0;
        let mut states_to_remove = Vec::new();
        
        for (state_id, state) in &self.states {
            if state.is_expired() {
                states_to_remove.push(state_id.clone());
            }
        }
        
        for state_id in states_to_remove {
            self.states.remove(&state_id);
            removed_count += 1;
        }
        
        removed_count
    }
}

impl Default for OAuth2Manager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_oauth2_manager_creation() {
        let manager = OAuth2Manager::new();
        assert_eq!(manager.list_providers().len(), 0);
    }
    
    #[tokio::test]
    async fn test_add_oauth2_provider() {
        let mut manager = OAuth2Manager::new();
        let provider = OAuth2Provider::google(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );
        
        manager.add_provider(provider);
        assert_eq!(manager.list_providers().len(), 1);
        assert!(manager.get_provider("google").is_some());
    }
    
    #[tokio::test]
    async fn test_start_authorization() {
        let mut manager = OAuth2Manager::new();
        let provider = OAuth2Provider::google(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );
        
        manager.add_provider(provider);
        
        let auth_url = manager.start_authorization("google", None).await.unwrap();
        assert!(auth_url.contains("accounts.google.com"));
        assert!(auth_url.contains("client_id=client_id"));
    }
    
    #[tokio::test]
    async fn test_oauth2_state_expiration() {
        let state = OAuth2State::new("google".to_string(), None);
        assert!(!state.is_expired());
        
        // Create an expired state
        let mut expired_state = state;
        expired_state.expires_at = SystemTime::now() - Duration::from_secs(1);
        assert!(expired_state.is_expired());
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_states() {
        let mut manager = OAuth2Manager::new();
        
        // Add an expired state manually
        let mut expired_state = OAuth2State::new("google".to_string(), None);
        expired_state.expires_at = SystemTime::now() - Duration::from_secs(1);
        manager.states.insert("expired".to_string(), expired_state);
        
        let removed_count = manager.cleanup_expired_states().await;
        assert_eq!(removed_count, 1);
        assert!(!manager.states.contains_key("expired"));
    }
}
