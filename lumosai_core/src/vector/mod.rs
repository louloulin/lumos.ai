//! Vector storage module - now using unified lumos-vector-core architecture
//!
//! This module provides backward compatibility while using the new unified
//! vector storage architecture under the hood.

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::error::{Error, Result as LumosResult};

// Re-export new unified architecture
pub use lumosai_vector::prelude::*;

// Convert from lumosai_core::Error to VectorError
impl From<Error> for VectorError {
    fn from(err: Error) -> Self {
        match err {
            Error::InvalidInput(msg) => VectorError::InvalidConfig(msg),
            Error::Storage(msg) => VectorError::StorageBackend(msg),
            Error::Configuration(msg) => VectorError::InvalidConfig(msg),
            Error::NotFound(msg) => VectorError::IndexNotFound(msg),
            Error::AlreadyExists(msg) => VectorError::IndexAlreadyExists(msg),
            Error::Timeout(msg) => VectorError::QueryTimeout { seconds: 30 },
            Error::Internal(msg) => VectorError::Internal(msg),
            _ => VectorError::Internal(err.to_string()),
        }
    }
}
pub use lumosai_vector::memory::MemoryVectorStorage as NewMemoryVectorStorage;

/// Legacy vector similarity metrics (for backward compatibility)
/// Use lumos_vector_core::SimilarityMetric for new code
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SimilarityMetric {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Dot product
    DotProduct,
}

impl From<SimilarityMetric> for lumosai_vector::SimilarityMetric {
    fn from(legacy: SimilarityMetric) -> Self {
        match legacy {
            SimilarityMetric::Cosine => lumosai_vector::SimilarityMetric::Cosine,
            SimilarityMetric::Euclidean => lumosai_vector::SimilarityMetric::Euclidean,
            SimilarityMetric::DotProduct => lumosai_vector::SimilarityMetric::DotProduct,
        }
    }
}

impl From<lumosai_vector::SimilarityMetric> for SimilarityMetric {
    fn from(new: lumosai_vector::SimilarityMetric) -> Self {
        match new {
            lumosai_vector::SimilarityMetric::Cosine => SimilarityMetric::Cosine,
            lumosai_vector::SimilarityMetric::Euclidean => SimilarityMetric::Euclidean,
            lumosai_vector::SimilarityMetric::DotProduct => SimilarityMetric::DotProduct,
            _ => SimilarityMetric::Cosine, // Default fallback
        }
    }
}

/// Legacy vector index statistics (for backward compatibility)
/// Use lumos_vector_core::IndexInfo for new code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of dimensions in the vectors
    pub dimension: usize,
    /// Number of vectors in the index
    pub count: usize,
    /// Similarity metric used
    pub metric: SimilarityMetric,
}

impl From<lumosai_vector::IndexInfo> for IndexStats {
    fn from(info: lumosai_vector::IndexInfo) -> Self {
        Self {
            dimension: info.dimension,
            count: info.vector_count,
            metric: info.metric.into(),
        }
    }
}

/// Legacy query result from vector search (for backward compatibility)
/// Use lumos_vector_core::SearchResult for new code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Vector ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Vector data (optional)
    pub vector: Option<Vec<f32>>,
    /// Associated metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl From<lumosai_vector::SearchResult> for QueryResult {
    fn from(result: lumosai_vector::SearchResult) -> Self {
        Self {
            id: result.id,
            score: result.score,
            vector: result.vector,
            metadata: result.metadata.map(|m| convert_metadata_to_json(m)),
        }
    }
}

/// Convert new metadata format to legacy JSON format
fn convert_metadata_to_json(metadata: lumosai_vector::Metadata) -> HashMap<String, serde_json::Value> {
    metadata.into_iter()
        .map(|(k, v)| (k, convert_metadata_value_to_json(v)))
        .collect()
}

/// Convert metadata value to JSON value
fn convert_metadata_value_to_json(value: lumosai_vector::MetadataValue) -> serde_json::Value {
    match value {
        lumosai_vector::MetadataValue::String(s) => serde_json::Value::String(s),
        lumosai_vector::MetadataValue::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
        lumosai_vector::MetadataValue::Float(f) => {
            serde_json::Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0)))
        },
        lumosai_vector::MetadataValue::Boolean(b) => serde_json::Value::Bool(b),
        lumosai_vector::MetadataValue::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(convert_metadata_value_to_json).collect())
        },
        lumosai_vector::MetadataValue::Object(obj) => {
            serde_json::Value::Object(obj.into_iter().map(|(k, v)| (k, convert_metadata_value_to_json(v))).collect())
        },
        lumosai_vector::MetadataValue::Null => serde_json::Value::Null,
    }
}

pub mod filter;
// pub use filter::FilterCondition; // Temporarily disabled to avoid conflict with types::FilterCondition

/// Re-export types
pub use types::{Vector, IndexConfig, EmbeddingService, FilterCondition, SearchParams};

/// Vector storage trait
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Create a new vector index
    async fn create_index(
        &self,
        index_name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> std::result::Result<(), VectorError>;

    /// List all vector indexes
    async fn list_indexes(&self) -> std::result::Result<Vec<String>, VectorError>;

    /// Get index statistics
    async fn describe_index(&self, index_name: &str) -> std::result::Result<IndexStats, VectorError>;

    /// Delete an index
    async fn delete_index(&self, index_name: &str) -> std::result::Result<(), VectorError>;

    /// Insert or update vectors
    async fn upsert(
        &self,
        index_name: &str,
        vectors: Vec<Vec<f32>>,
        ids: Option<Vec<String>>,
        metadata: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> std::result::Result<Vec<String>, VectorError>;

    /// Query vectors by similarity
    async fn query(
        &self,
        index_name: &str,
        query_vector: Vec<f32>,
        top_k: usize,
        filter: Option<FilterCondition>,
        include_vectors: bool,
    ) -> std::result::Result<Vec<QueryResult>, VectorError>;

    /// Update vector by ID
    async fn update_by_id(
        &self,
        index_name: &str,
        id: &str,
        vector: Option<Vec<f32>>,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<()>;

    /// Delete vector by ID
    async fn delete_by_id(&self, index_name: &str, id: &str) -> Result<()>;
}

pub mod memory;
pub mod types;
// pub mod adapter; // Temporarily disabled due to dependency conflicts

// Re-export memory implementation
pub use memory::MemoryVectorStorage;
// pub use adapter::VectorStorageAdapter;

/// Create a new memory vector storage instance
pub fn create_memory_vector_storage() -> MemoryVectorStorage {
    MemoryVectorStorage::new(1536, None)
}

/// Simple embedding module
pub mod embedding;

#[cfg(feature = "vector_sqlite")]
pub mod sqlite;

#[cfg(feature = "vector_sqlite")]
pub use self::sqlite::SqliteVectorStorage;

/// Vector storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorStorageConfig {
    /// In-memory vector storage
    Memory {
        /// 维度
        dimensions: usize,
        /// 内存容量
        capacity: Option<usize>,
    },
    /// SQLite vector storage
    #[cfg(feature = "vector_sqlite")]
    Sqlite {
        /// Path to SQLite database file
        db_path: String,
        /// Whether to use in-memory SQLite database
        in_memory: bool,
    },
    /// Qdrant vector storage
    Qdrant {
        /// Qdrant server URL
        url: String,
        /// Optional API key
        api_key: Option<String>,
    },
    /// Weaviate vector storage
    Weaviate {
        /// Weaviate server URL
        url: String,
        /// Optional API key
        api_key: Option<String>,
    },
    /// MongoDB vector storage
    MongoDB {
        /// MongoDB connection string
        connection_string: String,
        /// Database name
        database: String,
    },
    /// PostgreSQL vector storage with pgvector
    PostgreSQL {
        /// PostgreSQL connection string
        connection_string: String,
        /// Database name
        database: String,
    },
    /// Other type
    Other(Value),
}

impl Default for VectorStorageConfig {
    fn default() -> Self {
        Self::Memory {
            dimensions: 1536,
            capacity: None,
        }
    }
}

/// Create a vector storage instance from configuration
pub fn create_vector_storage(config: Option<VectorStorageConfig>) -> Result<Box<dyn VectorStorage>> {
    let config = config.unwrap_or_else(VectorStorageConfig::default);

    match config {
        VectorStorageConfig::Memory { dimensions, capacity } => {
            let storage = memory::MemoryVectorStorage::new(dimensions, capacity);
            Ok(Box::new(storage))
        },
        #[cfg(feature = "vector_sqlite")]
        VectorStorageConfig::Sqlite { db_path, in_memory } => {
            if in_memory {
                Ok(Box::new(self::sqlite::create_sqlite_vector_storage_in_memory()?))
            } else {
                Ok(Box::new(self::sqlite::create_sqlite_vector_storage(db_path)?))
            }
        },
        VectorStorageConfig::Qdrant { url, api_key } => {
            #[cfg(feature = "qdrant")]
            {
                use crate::vector::qdrant::QdrantVectorStorage;
                let mut config = crate::vector::qdrant::QdrantConfig::new(&url);
                if let Some(key) = api_key {
                    config = config.with_api_key(key);
                }
                let storage = QdrantVectorStorage::with_config(config).await
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok(Box::new(storage))
            }
            #[cfg(not(feature = "qdrant"))]
            {
                Err(VectorError::InvalidConfig("Qdrant support not enabled. Enable 'qdrant' feature".to_string()))
            }
        },
        VectorStorageConfig::Weaviate { url, api_key } => {
            #[cfg(feature = "weaviate")]
            {
                use crate::vector::weaviate::WeaviateVectorStorage;
                let mut config = crate::vector::weaviate::WeaviateConfig::new(&url);
                if let Some(key) = api_key {
                    config = config.with_api_key(key);
                }
                let storage = WeaviateVectorStorage::with_config(config).await
                    .map_err(|e| Error::Storage(e.to_string()))?;
                Ok(Box::new(storage))
            }
            #[cfg(not(feature = "weaviate"))]
            {
                Err(VectorError::InvalidConfig("Weaviate support not enabled. Enable 'weaviate' feature".to_string()))
            }
        },
        VectorStorageConfig::MongoDB { connection_string: _, database: _ } => {
            Err(VectorError::NotSupported("MongoDB support temporarily disabled due to dependency conflicts".to_string()))
        },
        VectorStorageConfig::PostgreSQL { connection_string: _, database: _ } => {
            Err(VectorError::NotSupported("PostgreSQL support temporarily disabled due to dependency conflicts".to_string()))
        },
        _ => Err(VectorError::InvalidConfig("Unsupported vector storage configuration".to_string())),
    }
}

/// Re-export embedding service functions
pub use embedding::create_random_embedding;

/// 文档表示，包含内容、元数据和可选的向量表示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// 文档唯一标识符
    pub id: String,
    /// 文档内容
    pub content: String,
    /// 文档元数据
    pub metadata: HashMap<String, Value>,
    /// 文档的向量表示（嵌入）
    pub embedding: Vec<f32>,
}

impl Document {
    /// 创建新文档
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: HashMap::new(),
            embedding: Vec::new(),
        }
    }
    
    /// 添加元数据
    pub fn add_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }
    
    /// 添加向量表示
    pub fn add_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = embedding;
    }
    
    /// 计算与另一文档的余弦相似度
    pub fn cosine_similarity(&self, other: &Document) -> Option<f32> {
        if self.embedding.len() != other.embedding.len() {
            return None;
        }
        
        let mut dot_product = 0.0;
        let mut magnitude1 = 0.0;
        let mut magnitude2 = 0.0;
        
        for (v1, v2) in self.embedding.iter().zip(other.embedding.iter()) {
            dot_product += v1 * v2;
            magnitude1 += v1 * v1;
            magnitude2 += v2 * v2;
        }
        
        magnitude1 = magnitude1.sqrt();
        magnitude2 = magnitude2.sqrt();
        
        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return None;
        }
        
        Some(dot_product / (magnitude1 * magnitude2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_vector_storage_factory() {
        // Test default memory storage
        let config = VectorStorageConfig::default();
        let storage = create_vector_storage(Some(config)).unwrap();
        
        // Create test index
        storage.create_index("test_factory", 3, None).await.unwrap();
        
        // Insert some vectors
        let vectors = vec![vec![1.0, 2.0, 3.0]];
        let ids = storage.upsert("test_factory", vectors, None, None).await.unwrap();
        
        // Verify index was created
        let indexes = storage.list_indexes().await.unwrap();
        assert!(indexes.contains(&"test_factory".to_string()));
        
        // Verify stats
        let stats = storage.describe_index("test_factory").await.unwrap();
        assert_eq!(stats.dimension, 3);
        assert_eq!(stats.count, 1);
        
        // Verify we can query
        let results = storage.query(
            "test_factory",
            vec![1.0, 2.0, 3.0],
            1,
            None,
            true
        ).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, ids[0]);
        
        // Clean up
        storage.delete_index("test_factory").await.unwrap();
        
        #[cfg(feature = "vector_sqlite")]
        {
            // Test SQLite in-memory storage
            let sqlite_config = VectorStorageConfig::Sqlite {
                db_path: "".to_string(),
                in_memory: true,
            };
            
            let sqlite_storage = create_vector_storage(Some(sqlite_config)).unwrap();
            
            // Create test index
            sqlite_storage.create_index("sqlite_test", 3, None).await.unwrap();
            
            // Verify index was created
            let indexes = sqlite_storage.list_indexes().await.unwrap();
            assert!(indexes.contains(&"sqlite_test".to_string()));
            
            // Clean up
            sqlite_storage.delete_index("sqlite_test").await.unwrap();
        }
    }
} 