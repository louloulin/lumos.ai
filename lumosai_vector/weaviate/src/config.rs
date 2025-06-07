//! Weaviate configuration

use serde::{Deserialize, Serialize};
use url::Url;
use crate::error::{WeaviateError, WeaviateResult};

/// Configuration for Weaviate vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaviateConfig {
    /// Weaviate server URL
    pub url: String,
    
    /// Optional API key for authentication
    pub api_key: Option<String>,
    
    /// Optional OIDC token for authentication
    pub oidc_token: Option<String>,
    
    /// Class name prefix for multi-tenancy
    pub class_prefix: Option<String>,
    
    /// Default tenant name
    pub tenant: Option<String>,
    
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Whether to create schema automatically
    pub auto_schema: bool,
    
    /// Default vectorizer module
    pub vectorizer: Option<String>,
}

impl Default for WeaviateConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".to_string(),
            api_key: None,
            oidc_token: None,
            class_prefix: None,
            tenant: None,
            timeout_seconds: 30,
            batch_size: 100,
            auto_schema: true,
            vectorizer: None,
        }
    }
}

impl WeaviateConfig {
    /// Create a new Weaviate configuration
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            ..Default::default()
        }
    }
    
    /// Set API key for authentication
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    
    /// Set OIDC token for authentication
    pub fn with_oidc_token(mut self, token: String) -> Self {
        self.oidc_token = Some(token);
        self
    }
    
    /// Set class prefix for multi-tenancy
    pub fn with_class_prefix(mut self, prefix: String) -> Self {
        self.class_prefix = Some(prefix);
        self
    }
    
    /// Set tenant name
    pub fn with_tenant(mut self, tenant: String) -> Self {
        self.tenant = Some(tenant);
        self
    }
    
    /// Set request timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
    
    /// Set batch size for bulk operations
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }
    
    /// Enable or disable automatic schema creation
    pub fn with_auto_schema(mut self, auto: bool) -> Self {
        self.auto_schema = auto;
        self
    }
    
    /// Set default vectorizer module
    pub fn with_vectorizer(mut self, vectorizer: String) -> Self {
        self.vectorizer = Some(vectorizer);
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> WeaviateResult<()> {
        // Validate URL
        Url::parse(&self.url)
            .map_err(|e| WeaviateError::Config(format!("Invalid URL: {}", e)))?;
        
        // Validate batch size
        if self.batch_size == 0 {
            return Err(WeaviateError::Config("Batch size must be greater than 0".to_string()));
        }
        
        // Validate timeout
        if self.timeout_seconds == 0 {
            return Err(WeaviateError::Config("Timeout must be greater than 0".to_string()));
        }
        
        Ok(())
    }
    
    /// Get the full class name with prefix
    pub fn class_name(&self, name: &str) -> String {
        if let Some(prefix) = &self.class_prefix {
            format!("{}_{}", prefix, name)
        } else {
            name.to_string()
        }
    }
    
    /// Get the base URL for API requests
    pub fn api_url(&self) -> String {
        format!("{}/v1", self.url.trim_end_matches('/'))
    }
}
