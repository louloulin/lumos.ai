use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::error::{Error, Result};

/// Vector similarity metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SimilarityMetric {
    /// Cosine similarity
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Dot product
    DotProduct,
}

/// Vector index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of dimensions in the vectors
    pub dimension: usize,
    /// Number of vectors in the index
    pub count: usize,
    /// Similarity metric used
    pub metric: SimilarityMetric,
}

/// Query result from vector search
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

pub mod filter;
pub use filter::FilterCondition;

/// Re-export types
pub use types::{Vector, IndexConfig, EmbeddingService};

/// Vector storage trait
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Create a new vector index
    async fn create_index(
        &self,
        index_name: &str,
        dimension: usize,
        metric: Option<SimilarityMetric>,
    ) -> Result<()>;

    /// List all vector indexes
    async fn list_indexes(&self) -> Result<Vec<String>>;

    /// Get index statistics
    async fn describe_index(&self, index_name: &str) -> Result<IndexStats>;

    /// Delete an index
    async fn delete_index(&self, index_name: &str) -> Result<()>;

    /// Insert or update vectors
    async fn upsert(
        &self,
        index_name: &str,
        vectors: Vec<Vec<f32>>,
        ids: Option<Vec<String>>,
        metadata: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> Result<Vec<String>>;

    /// Query vectors by similarity
    async fn query(
        &self,
        index_name: &str,
        query_vector: Vec<f32>,
        top_k: usize,
        filter: Option<FilterCondition>,
        include_vectors: bool,
    ) -> Result<Vec<QueryResult>>;

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

// Re-export memory implementation
pub use memory::MemoryVectorStorage;

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
    let config = config.unwrap_or_else(|| VectorStorageConfig::default());
    
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
        }
        _ => Err(Error::InvalidInput("Unsupported vector storage configuration".to_string())),
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
        let storage = create_vector_storage(config).unwrap();
        
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
            
            let sqlite_storage = create_vector_storage(sqlite_config).unwrap();
            
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