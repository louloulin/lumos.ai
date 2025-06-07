//! Configuration for Qdrant vector storage

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for Qdrant vector storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    /// Qdrant server URL
    pub url: String,
    
    /// Optional API key for authentication
    pub api_key: Option<String>,
    
    /// Connection timeout
    pub timeout: Duration,
    
    /// Maximum number of connections in the pool
    pub max_connections: usize,
    
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Enable TLS
    pub tls: bool,
    
    /// Collection prefix for multi-tenancy
    pub collection_prefix: Option<String>,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:6334".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            max_connections: 10,
            batch_size: 100,
            tls: false,
            collection_prefix: None,
        }
    }
}

impl QdrantConfig {
    /// Create a new configuration with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
    
    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }
    
    /// Set the connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set the maximum number of connections
    pub fn with_max_connections(mut self, max_connections: usize) -> Self {
        self.max_connections = max_connections;
        self
    }
    
    /// Set the batch size for bulk operations
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
    
    /// Enable TLS
    pub fn with_tls(mut self, tls: bool) -> Self {
        self.tls = tls;
        self
    }
    
    /// Set the collection prefix
    pub fn with_collection_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.collection_prefix = Some(prefix.into());
        self
    }
    
    /// Get the full collection name with prefix
    pub fn collection_name(&self, name: &str) -> String {
        match &self.collection_prefix {
            Some(prefix) => format!("{}_{}", prefix, name),
            None => name.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = QdrantConfig::default();
        assert_eq!(config.url, "http://localhost:6334");
        assert_eq!(config.api_key, None);
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.batch_size, 100);
        assert!(!config.tls);
        assert_eq!(config.collection_prefix, None);
    }
    
    #[test]
    fn test_builder_pattern() {
        let config = QdrantConfig::new("http://example.com:6334")
            .with_api_key("test-key")
            .with_timeout(Duration::from_secs(60))
            .with_max_connections(20)
            .with_batch_size(200)
            .with_tls(true)
            .with_collection_prefix("test");
            
        assert_eq!(config.url, "http://example.com:6334");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.batch_size, 200);
        assert!(config.tls);
        assert_eq!(config.collection_prefix, Some("test".to_string()));
    }
    
    #[test]
    fn test_collection_name() {
        let config = QdrantConfig::default();
        assert_eq!(config.collection_name("test"), "test");
        
        let config = config.with_collection_prefix("prefix");
        assert_eq!(config.collection_name("test"), "prefix_test");
    }
}
