//! Milvus configuration module

use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::error::{MilvusError, MilvusResult};

/// Milvus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilvusConfig {
    /// Milvus endpoint URL
    pub endpoint: String,
    
    /// Database name
    pub database: String,
    
    /// Connection timeout
    pub timeout: Duration,
    
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    
    /// Index configuration
    pub index_config: IndexConfiguration,
    
    /// Performance tuning options
    pub performance: PerformanceConfig,
    
    /// Collection configuration
    pub collection_config: CollectionConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Username
    pub username: String,
    
    /// Password
    pub password: String,
    
    /// Token (for token-based authentication)
    pub token: Option<String>,
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfiguration {
    /// Default index type for new collections
    pub default_index_type: IndexType,
    
    /// Index parameters
    pub index_params: IndexParams,
    
    /// Auto-create indexes for new collections
    pub auto_create_index: bool,
    
    /// Index build threshold (minimum number of entities before building index)
    pub build_threshold: usize,
}

/// Index type supported by Milvus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    /// FLAT index (brute force)
    FLAT,
    
    /// IVF_FLAT index
    IVF_FLAT,
    
    /// IVF_SQ8 index (scalar quantization)
    IVF_SQ8,
    
    /// IVF_PQ index (product quantization)
    IVF_PQ,
    
    /// HNSW index
    HNSW,
    
    /// ANNOY index
    ANNOY,
    
    /// AUTOINDEX (let Milvus choose)
    AUTOINDEX,
}

/// Index parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexParams {
    /// Number of clusters for IVF index
    pub nlist: Option<usize>,
    
    /// Number of sub-quantizers for PQ
    pub m: Option<usize>,
    
    /// Number of bits per sub-quantizer
    pub nbits: Option<usize>,
    
    /// HNSW M parameter (number of connections)
    pub hnsw_m: Option<usize>,
    
    /// HNSW ef_construction parameter
    pub ef_construction: Option<usize>,
    
    /// ANNOY number of trees
    pub n_trees: Option<usize>,
    
    /// Search parameter ef for HNSW
    pub ef: Option<usize>,
    
    /// Search parameter nprobe for IVF
    pub nprobe: Option<usize>,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Number of parallel requests
    pub max_parallel_requests: usize,
    
    /// Request timeout
    pub request_timeout: Duration,
    
    /// Retry configuration
    pub retry_config: RetryConfig,
    
    /// Connection pool size
    pub connection_pool_size: usize,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,
    
    /// Initial retry delay
    pub initial_delay: Duration,
    
    /// Maximum retry delay
    pub max_delay: Duration,
    
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

/// Collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    /// Default consistency level
    pub consistency_level: ConsistencyLevel,
    
    /// Shards number for new collections
    pub shards_num: usize,
    
    /// Replica number
    pub replica_number: usize,
    
    /// Resource groups
    pub resource_groups: Vec<String>,
}

/// Consistency level for Milvus operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Strong consistency
    Strong,
    
    /// Session consistency
    Session,
    
    /// Bounded staleness
    Bounded,
    
    /// Eventually consistent
    Eventually,
}

impl Default for MilvusConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:19530".to_string(),
            database: "default".to_string(),
            timeout: Duration::from_secs(30),
            auth: None,
            index_config: IndexConfiguration::default(),
            performance: PerformanceConfig::default(),
            collection_config: CollectionConfig::default(),
        }
    }
}

impl Default for IndexConfiguration {
    fn default() -> Self {
        Self {
            default_index_type: IndexType::AUTOINDEX,
            index_params: IndexParams::default(),
            auto_create_index: true,
            build_threshold: 1000,
        }
    }
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            nlist: Some(1024),
            m: Some(8),
            nbits: Some(8),
            hnsw_m: Some(16),
            ef_construction: Some(200),
            n_trees: Some(8),
            ef: Some(64),
            nprobe: Some(10),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            max_parallel_requests: 10,
            request_timeout: Duration::from_secs(60),
            retry_config: RetryConfig::default(),
            connection_pool_size: 10,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            consistency_level: ConsistencyLevel::Session,
            shards_num: 1,
            replica_number: 1,
            resource_groups: vec!["default".to_string()],
        }
    }
}

impl MilvusConfig {
    /// Create a new Milvus configuration with the specified endpoint
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            ..Default::default()
        }
    }
    
    /// Set the database name
    pub fn with_database(mut self, database: &str) -> Self {
        self.database = database.to_string();
        self
    }
    
    /// Set authentication credentials
    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.auth = Some(AuthConfig {
            username: username.to_string(),
            password: password.to_string(),
            token: None,
        });
        self
    }
    
    /// Set authentication token
    pub fn with_token(mut self, token: &str) -> Self {
        if let Some(ref mut auth) = self.auth {
            auth.token = Some(token.to_string());
        } else {
            self.auth = Some(AuthConfig {
                username: String::new(),
                password: String::new(),
                token: Some(token.to_string()),
            });
        }
        self
    }
    
    /// Set connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set default index type
    pub fn with_default_index_type(mut self, index_type: IndexType) -> Self {
        self.index_config.default_index_type = index_type;
        self
    }
    
    /// Set batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.performance.batch_size = batch_size;
        self
    }
    
    /// Set consistency level
    pub fn with_consistency_level(mut self, level: ConsistencyLevel) -> Self {
        self.collection_config.consistency_level = level;
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> MilvusResult<()> {
        if self.endpoint.is_empty() {
            return Err(MilvusError::InvalidConfiguration("Endpoint cannot be empty".to_string()));
        }
        
        if self.database.is_empty() {
            return Err(MilvusError::InvalidConfiguration("Database name cannot be empty".to_string()));
        }
        
        if self.timeout.as_secs() == 0 {
            return Err(MilvusError::InvalidConfiguration("Timeout must be greater than 0".to_string()));
        }
        
        if self.performance.batch_size == 0 {
            return Err(MilvusError::InvalidConfiguration("Batch size must be greater than 0".to_string()));
        }
        
        if self.collection_config.shards_num == 0 {
            return Err(MilvusError::InvalidConfiguration("Shards number must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

/// Builder for Milvus configuration
pub struct MilvusConfigBuilder {
    config: MilvusConfig,
}

impl MilvusConfigBuilder {
    /// Create a new configuration builder
    pub fn new(endpoint: &str) -> Self {
        Self {
            config: MilvusConfig::new(endpoint),
        }
    }
    
    /// Set the database name
    pub fn database(mut self, database: &str) -> Self {
        self.config.database = database.to_string();
        self
    }
    
    /// Set authentication credentials
    pub fn auth(mut self, username: &str, password: &str) -> Self {
        self.config.auth = Some(AuthConfig {
            username: username.to_string(),
            password: password.to_string(),
            token: None,
        });
        self
    }
    
    /// Set authentication token
    pub fn token(mut self, token: &str) -> Self {
        if let Some(ref mut auth) = self.config.auth {
            auth.token = Some(token.to_string());
        } else {
            self.config.auth = Some(AuthConfig {
                username: String::new(),
                password: String::new(),
                token: Some(token.to_string()),
            });
        }
        self
    }
    
    /// Set connection timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }
    
    /// Set default index type
    pub fn default_index_type(mut self, index_type: IndexType) -> Self {
        self.config.index_config.default_index_type = index_type;
        self
    }
    
    /// Set batch size
    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.config.performance.batch_size = batch_size;
        self
    }
    
    /// Set consistency level
    pub fn consistency_level(mut self, level: ConsistencyLevel) -> Self {
        self.config.collection_config.consistency_level = level;
        self
    }
    
    /// Set shards number
    pub fn shards_num(mut self, shards_num: usize) -> Self {
        self.config.collection_config.shards_num = shards_num;
        self
    }
    
    /// Set replica number
    pub fn replica_number(mut self, replica_number: usize) -> Self {
        self.config.collection_config.replica_number = replica_number;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> MilvusResult<MilvusConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for MilvusConfigBuilder {
    fn default() -> Self {
        Self::new("http://localhost:19530")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = MilvusConfig::default();
        assert_eq!(config.endpoint, "http://localhost:19530");
        assert_eq!(config.database, "default");
        assert_eq!(config.performance.batch_size, 1000);
    }
    
    #[test]
    fn test_config_builder() {
        let config = MilvusConfigBuilder::new("http://localhost:19530")
            .database("test_db")
            .auth("user", "pass")
            .batch_size(500)
            .consistency_level(ConsistencyLevel::Strong)
            .build()
            .unwrap();
        
        assert_eq!(config.endpoint, "http://localhost:19530");
        assert_eq!(config.database, "test_db");
        assert_eq!(config.performance.batch_size, 500);
        assert!(matches!(config.collection_config.consistency_level, ConsistencyLevel::Strong));
        assert!(config.auth.is_some());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = MilvusConfig::default();
        assert!(config.validate().is_ok());
        
        config.endpoint = "".to_string();
        assert!(config.validate().is_err());
        
        config.endpoint = "http://localhost:19530".to_string();
        config.performance.batch_size = 0;
        assert!(config.validate().is_err());
    }
}
