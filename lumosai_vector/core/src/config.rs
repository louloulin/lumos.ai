//! Configuration types for the Lumos vector storage system

use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::types::{MetadataValue, SimilarityMetric};

/// Storage backend configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StorageConfig {
    /// In-memory storage configuration
    Memory {
        /// Initial capacity hint
        initial_capacity: Option<usize>,
        /// Maximum number of vectors
        max_vectors: Option<usize>,
    },
    
    /// SQLite storage configuration
    Sqlite {
        /// Database file path (":memory:" for in-memory)
        database_path: String,
        /// Connection pool size
        pool_size: Option<usize>,
        /// Enable WAL mode
        wal_mode: bool,
        /// Synchronous mode
        synchronous: SqliteSynchronous,
    },
    
    /// Qdrant storage configuration
    Qdrant {
        /// Qdrant server URL
        url: String,
        /// API key for authentication
        api_key: Option<String>,
        /// Collection name
        collection_name: String,
        /// Connection timeout
        timeout: Option<Duration>,
        /// Enable TLS
        tls: bool,
    },
    
    /// MongoDB storage configuration
    MongoDB {
        /// MongoDB connection string
        connection_string: String,
        /// Database name
        database: String,
        /// Collection name
        collection: String,
        /// Connection timeout
        timeout: Option<Duration>,
    },
    
    /// Custom storage configuration
    Custom {
        /// Backend name
        backend: String,
        /// Custom configuration options
        options: HashMap<String, MetadataValue>,
    },
}

/// SQLite synchronous mode
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SqliteSynchronous {
    Off,
    Normal,
    Full,
    Extra,
}

impl Default for SqliteSynchronous {
    fn default() -> Self {
        SqliteSynchronous::Normal
    }
}

/// Embedding model configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EmbeddingConfig {
    /// OpenAI embedding configuration
    OpenAI {
        /// API key
        api_key: String,
        /// Model name (e.g., "text-embedding-3-small")
        model: String,
        /// API base URL (optional)
        base_url: Option<String>,
        /// Organization ID (optional)
        organization: Option<String>,
        /// Request timeout
        timeout: Option<Duration>,
        /// Max retries
        max_retries: u32,
    },
    
    /// Ollama embedding configuration
    Ollama {
        /// Ollama server URL
        url: String,
        /// Model name
        model: String,
        /// Request timeout
        timeout: Option<Duration>,
    },
    
    /// Local model configuration
    Local {
        /// Model path or identifier
        model_path: String,
        /// Device to run on (cpu, cuda, etc.)
        device: String,
        /// Model options
        options: HashMap<String, MetadataValue>,
    },
    
    /// Custom embedding configuration
    Custom {
        /// Provider name
        provider: String,
        /// Custom configuration options
        options: HashMap<String, MetadataValue>,
    },
}

/// Index creation configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndexCreateConfig {
    /// Index name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric
    pub metric: SimilarityMetric,
    /// Index-specific options
    pub options: IndexOptions,
}

/// Index-specific options
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndexOptions {
    /// Enable approximate nearest neighbor search
    pub approximate: bool,
    /// Number of trees for approximate search (if supported)
    pub num_trees: Option<usize>,
    /// Search accuracy vs speed tradeoff (0.0 to 1.0)
    pub accuracy: Option<f32>,
    /// Maximum number of vectors in the index
    pub max_vectors: Option<usize>,
    /// Enable compression
    pub compression: bool,
    /// Custom backend-specific options
    pub custom: HashMap<String, MetadataValue>,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self {
            approximate: false,
            num_trees: None,
            accuracy: None,
            max_vectors: None,
            compression: false,
            custom: HashMap::new(),
        }
    }
}

/// Search configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchConfig {
    /// Search timeout
    pub timeout: Option<Duration>,
    /// Enable approximate search
    pub approximate: bool,
    /// Search accuracy (for approximate search)
    pub accuracy: Option<f32>,
    /// Enable result caching
    pub cache_results: bool,
    /// Cache TTL
    pub cache_ttl: Option<Duration>,
    /// Custom search options
    pub custom: HashMap<String, MetadataValue>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            approximate: false,
            accuracy: None,
            cache_results: false,
            cache_ttl: None,
            custom: HashMap::new(),
        }
    }
}

/// Complete vector storage system configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VectorSystemConfig {
    /// Storage backend configuration
    pub storage: StorageConfig,
    /// Embedding model configuration (optional)
    pub embedding: Option<EmbeddingConfig>,
    /// Default search configuration
    pub search: SearchConfig,
    /// System-wide options
    pub options: HashMap<String, MetadataValue>,
}

impl VectorSystemConfig {
    /// Create a new configuration with memory storage
    pub fn memory() -> Self {
        Self {
            storage: StorageConfig::Memory {
                initial_capacity: None,
                max_vectors: None,
            },
            embedding: None,
            search: SearchConfig::default(),
            options: HashMap::new(),
        }
    }
    
    /// Create a new configuration with SQLite storage
    pub fn sqlite(database_path: impl Into<String>) -> Self {
        Self {
            storage: StorageConfig::Sqlite {
                database_path: database_path.into(),
                pool_size: None,
                wal_mode: true,
                synchronous: SqliteSynchronous::default(),
            },
            embedding: None,
            search: SearchConfig::default(),
            options: HashMap::new(),
        }
    }
    
    /// Create a new configuration with Qdrant storage
    pub fn qdrant(url: impl Into<String>, collection: impl Into<String>) -> Self {
        Self {
            storage: StorageConfig::Qdrant {
                url: url.into(),
                api_key: None,
                collection_name: collection.into(),
                timeout: None,
                tls: false,
            },
            embedding: None,
            search: SearchConfig::default(),
            options: HashMap::new(),
        }
    }
    
    /// Set the embedding configuration
    pub fn with_embedding(mut self, embedding: EmbeddingConfig) -> Self {
        self.embedding = Some(embedding);
        self
    }
    
    /// Set the search configuration
    pub fn with_search_config(mut self, search: SearchConfig) -> Self {
        self.search = search;
        self
    }
    
    /// Add a system option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Builder for creating storage configurations
pub struct StorageConfigBuilder {
    config: StorageConfig,
}

impl StorageConfigBuilder {
    /// Create a memory storage builder
    pub fn memory() -> Self {
        Self {
            config: StorageConfig::Memory {
                initial_capacity: None,
                max_vectors: None,
            },
        }
    }
    
    /// Create a SQLite storage builder
    pub fn sqlite(database_path: impl Into<String>) -> Self {
        Self {
            config: StorageConfig::Sqlite {
                database_path: database_path.into(),
                pool_size: None,
                wal_mode: true,
                synchronous: SqliteSynchronous::default(),
            },
        }
    }
    
    /// Create a Qdrant storage builder
    pub fn qdrant(url: impl Into<String>, collection: impl Into<String>) -> Self {
        Self {
            config: StorageConfig::Qdrant {
                url: url.into(),
                api_key: None,
                collection_name: collection.into(),
                timeout: None,
                tls: false,
            },
        }
    }
    
    /// Set initial capacity for memory storage
    pub fn with_initial_capacity(mut self, capacity: usize) -> Self {
        if let StorageConfig::Memory { ref mut initial_capacity, .. } = self.config {
            *initial_capacity = Some(capacity);
        }
        self
    }
    
    /// Set API key for Qdrant
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        if let StorageConfig::Qdrant { api_key: ref mut key, .. } = self.config {
            *key = Some(api_key.into());
        }
        self
    }
    
    /// Enable TLS for Qdrant
    pub fn with_tls(mut self, enable: bool) -> Self {
        if let StorageConfig::Qdrant { ref mut tls, .. } = self.config {
            *tls = enable;
        }
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> StorageConfig {
        self.config
    }
}
