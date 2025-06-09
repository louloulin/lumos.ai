//! LanceDB configuration module

use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::error::{LanceDbError, LanceDbResult};

/// LanceDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanceDbConfig {
    /// Database URI (file path or cloud storage URL)
    pub uri: String,
    
    /// Connection timeout
    pub timeout: Option<Duration>,
    
    /// Maximum number of connections in the pool
    pub max_connections: Option<usize>,
    
    /// Enable write-ahead logging
    pub enable_wal: bool,
    
    /// Storage options for cloud providers
    pub storage_options: Option<StorageOptions>,
    
    /// Index configuration
    pub index_config: IndexConfiguration,
    
    /// Performance tuning options
    pub performance: PerformanceConfig,
}

/// Storage options for cloud providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptions {
    /// AWS S3 configuration
    pub s3: Option<S3Config>,
    
    /// Azure Blob Storage configuration
    pub azure: Option<AzureConfig>,
    
    /// Google Cloud Storage configuration
    pub gcs: Option<GcsConfig>,
}

/// AWS S3 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    /// AWS region
    pub region: String,
    
    /// Access key ID
    pub access_key_id: Option<String>,
    
    /// Secret access key
    pub secret_access_key: Option<String>,
    
    /// Session token for temporary credentials
    pub session_token: Option<String>,
    
    /// S3 endpoint URL (for S3-compatible services)
    pub endpoint_url: Option<String>,
}

/// Azure Blob Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    /// Storage account name
    pub account_name: String,
    
    /// Account key
    pub account_key: Option<String>,
    
    /// SAS token
    pub sas_token: Option<String>,
    
    /// Container name
    pub container_name: String,
}

/// Google Cloud Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsConfig {
    /// Service account key path
    pub service_account_key: Option<String>,
    
    /// Project ID
    pub project_id: String,
    
    /// Bucket name
    pub bucket_name: String,
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfiguration {
    /// Default index type for new tables
    pub default_index_type: IndexType,
    
    /// Index parameters
    pub index_params: IndexParams,
    
    /// Auto-create indexes for new tables
    pub auto_create_index: bool,
}

/// Index type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexType {
    /// IVF (Inverted File) index
    IVF,
    
    /// IVF with Product Quantization
    IVFPQ,
    
    /// Hierarchical Navigable Small World
    HNSW,
    
    /// Locality Sensitive Hashing
    LSH,
    
    /// No index (brute force search)
    None,
}

/// Index parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexParams {
    /// Number of clusters for IVF index
    pub num_clusters: Option<usize>,
    
    /// Number of sub-quantizers for PQ
    pub num_sub_quantizers: Option<usize>,
    
    /// Number of bits per sub-quantizer
    pub bits_per_sub_quantizer: Option<usize>,
    
    /// HNSW M parameter (number of connections)
    pub hnsw_m: Option<usize>,
    
    /// HNSW ef_construction parameter
    pub hnsw_ef_construction: Option<usize>,
    
    /// LSH number of hash tables
    pub lsh_num_tables: Option<usize>,
    
    /// LSH number of hash functions per table
    pub lsh_num_functions: Option<usize>,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Batch size for bulk operations
    pub batch_size: usize,
    
    /// Number of parallel threads for operations
    pub num_threads: Option<usize>,
    
    /// Memory limit for operations (in bytes)
    pub memory_limit: Option<usize>,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Compression level (0-9)
    pub compression_level: Option<u8>,
    
    /// Cache size for frequently accessed data
    pub cache_size: Option<usize>,
}

impl Default for LanceDbConfig {
    fn default() -> Self {
        Self {
            uri: "./lance_data".to_string(),
            timeout: Some(Duration::from_secs(30)),
            max_connections: Some(10),
            enable_wal: true,
            storage_options: None,
            index_config: IndexConfiguration::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for IndexConfiguration {
    fn default() -> Self {
        Self {
            default_index_type: IndexType::IVF,
            index_params: IndexParams::default(),
            auto_create_index: true,
        }
    }
}

impl Default for IndexParams {
    fn default() -> Self {
        Self {
            num_clusters: Some(256),
            num_sub_quantizers: Some(8),
            bits_per_sub_quantizer: Some(8),
            hnsw_m: Some(16),
            hnsw_ef_construction: Some(200),
            lsh_num_tables: Some(10),
            lsh_num_functions: Some(5),
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            num_threads: None, // Use system default
            memory_limit: None, // No limit
            enable_compression: true,
            compression_level: Some(6),
            cache_size: Some(1024 * 1024 * 100), // 100MB
        }
    }
}

impl LanceDbConfig {
    /// Create a new LanceDB configuration with the specified URI
    pub fn new(uri: &str) -> Self {
        Self {
            uri: uri.to_string(),
            ..Default::default()
        }
    }
    
    /// Create a configuration for local file storage
    pub fn local(path: &str) -> Self {
        Self::new(&format!("file://{}", path))
    }
    
    /// Create a configuration for AWS S3 storage
    pub fn s3(bucket: &str, region: &str) -> Self {
        let mut config = Self::new(&format!("s3://{}", bucket));
        config.storage_options = Some(StorageOptions {
            s3: Some(S3Config {
                region: region.to_string(),
                access_key_id: None,
                secret_access_key: None,
                session_token: None,
                endpoint_url: None,
            }),
            azure: None,
            gcs: None,
        });
        config
    }
    
    /// Create a configuration for Azure Blob Storage
    pub fn azure(account_name: &str, container_name: &str) -> Self {
        let mut config = Self::new(&format!("azure://{}/{}", account_name, container_name));
        config.storage_options = Some(StorageOptions {
            s3: None,
            azure: Some(AzureConfig {
                account_name: account_name.to_string(),
                account_key: None,
                sas_token: None,
                container_name: container_name.to_string(),
            }),
            gcs: None,
        });
        config
    }
    
    /// Create a configuration for Google Cloud Storage
    pub fn gcs(project_id: &str, bucket_name: &str) -> Self {
        let mut config = Self::new(&format!("gs://{}", bucket_name));
        config.storage_options = Some(StorageOptions {
            s3: None,
            azure: None,
            gcs: Some(GcsConfig {
                service_account_key: None,
                project_id: project_id.to_string(),
                bucket_name: bucket_name.to_string(),
            }),
        });
        config
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> LanceDbResult<()> {
        if self.uri.is_empty() {
            return Err(LanceDbError::InvalidConfiguration("URI cannot be empty".to_string()));
        }
        
        if let Some(timeout) = self.timeout {
            if timeout.as_secs() == 0 {
                return Err(LanceDbError::InvalidConfiguration("Timeout must be greater than 0".to_string()));
            }
        }
        
        if let Some(max_connections) = self.max_connections {
            if max_connections == 0 {
                return Err(LanceDbError::InvalidConfiguration("Max connections must be greater than 0".to_string()));
            }
        }
        
        if self.performance.batch_size == 0 {
            return Err(LanceDbError::InvalidConfiguration("Batch size must be greater than 0".to_string()));
        }
        
        Ok(())
    }
}

/// Builder for LanceDB configuration
pub struct LanceDbConfigBuilder {
    config: LanceDbConfig,
}

impl LanceDbConfigBuilder {
    /// Create a new configuration builder
    pub fn new(uri: &str) -> Self {
        Self {
            config: LanceDbConfig::new(uri),
        }
    }
    
    /// Set the connection timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = Some(timeout);
        self
    }
    
    /// Set the maximum number of connections
    pub fn max_connections(mut self, max_connections: usize) -> Self {
        self.config.max_connections = Some(max_connections);
        self
    }
    
    /// Enable or disable write-ahead logging
    pub fn enable_wal(mut self, enable: bool) -> Self {
        self.config.enable_wal = enable;
        self
    }
    
    /// Set the default index type
    pub fn default_index_type(mut self, index_type: IndexType) -> Self {
        self.config.index_config.default_index_type = index_type;
        self
    }
    
    /// Set the batch size for bulk operations
    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.config.performance.batch_size = batch_size;
        self
    }
    
    /// Set the number of threads for operations
    pub fn num_threads(mut self, num_threads: usize) -> Self {
        self.config.performance.num_threads = Some(num_threads);
        self
    }
    
    /// Enable or disable compression
    pub fn enable_compression(mut self, enable: bool) -> Self {
        self.config.performance.enable_compression = enable;
        self
    }
    
    /// Set the compression level
    pub fn compression_level(mut self, level: u8) -> Self {
        self.config.performance.compression_level = Some(level);
        self
    }
    
    /// Set the cache size
    pub fn cache_size(mut self, size: usize) -> Self {
        self.config.performance.cache_size = Some(size);
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> LanceDbResult<LanceDbConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for LanceDbConfigBuilder {
    fn default() -> Self {
        Self::new("./lance_data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = LanceDbConfig::default();
        assert_eq!(config.uri, "./lance_data");
        assert!(config.enable_wal);
        assert_eq!(config.performance.batch_size, 1000);
    }
    
    #[test]
    fn test_local_config() {
        let config = LanceDbConfig::local("/tmp/lance");
        assert_eq!(config.uri, "file:///tmp/lance");
    }
    
    #[test]
    fn test_s3_config() {
        let config = LanceDbConfig::s3("my-bucket", "us-west-2");
        assert_eq!(config.uri, "s3://my-bucket");
        assert!(config.storage_options.is_some());
    }
    
    #[test]
    fn test_config_builder() {
        let config = LanceDbConfigBuilder::new("./test")
            .timeout(Duration::from_secs(60))
            .batch_size(500)
            .enable_compression(false)
            .build()
            .unwrap();
        
        assert_eq!(config.uri, "./test");
        assert_eq!(config.timeout, Some(Duration::from_secs(60)));
        assert_eq!(config.performance.batch_size, 500);
        assert!(!config.performance.enable_compression);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = LanceDbConfig::default();
        assert!(config.validate().is_ok());
        
        config.uri = "".to_string();
        assert!(config.validate().is_err());
        
        config.uri = "./test".to_string();
        config.performance.batch_size = 0;
        assert!(config.validate().is_err());
    }
}
