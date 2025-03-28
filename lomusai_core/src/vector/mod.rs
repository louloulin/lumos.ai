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
    MemoryVectorStorage::new()
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
    Memory,
    /// SQLite vector storage
    #[cfg(feature = "vector_sqlite")]
    Sqlite {
        /// Path to SQLite database file
        db_path: String,
        /// Whether to use in-memory SQLite database
        in_memory: bool,
    },
}

impl Default for VectorStorageConfig {
    fn default() -> Self {
        Self::Memory
    }
}

/// Create a vector storage instance from configuration
pub fn create_vector_storage(config: VectorStorageConfig) -> Result<Box<dyn VectorStorage>> {
    match config {
        VectorStorageConfig::Memory => {
            Ok(Box::new(create_memory_vector_storage()))
        },
        #[cfg(feature = "vector_sqlite")]
        VectorStorageConfig::Sqlite { db_path, in_memory } => {
            if in_memory {
                Ok(Box::new(self::sqlite::create_sqlite_vector_storage_in_memory()?))
            } else {
                Ok(Box::new(self::sqlite::create_sqlite_vector_storage(db_path)?))
            }
        }
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
    pub embedding: Option<Vec<f32>>,
}

impl Document {
    /// 创建新文档
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            metadata: HashMap::new(),
            embedding: None,
        }
    }
    
    /// 添加元数据
    pub fn add_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }
    
    /// 添加向量表示
    pub fn add_embedding(&mut self, embedding: Vec<f32>) {
        self.embedding = Some(embedding);
    }
    
    /// 计算与另一文档的余弦相似度
    pub fn cosine_similarity(&self, other: &Document) -> Option<f32> {
        match (&self.embedding, &other.embedding) {
            (Some(vec1), Some(vec2)) => {
                if vec1.len() != vec2.len() {
                    return None;
                }
                
                let mut dot_product = 0.0;
                let mut magnitude1 = 0.0;
                let mut magnitude2 = 0.0;
                
                for (v1, v2) in vec1.iter().zip(vec2.iter()) {
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
            },
            _ => None,
        }
    }
}

/// 内存向量存储
pub struct MemoryVectorStore {
    documents: HashMap<String, Document>,
}

impl MemoryVectorStore {
    /// 创建新的内存向量存储
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }
    
    /// 添加文档
    pub fn add_document(&mut self, document: Document) {
        self.documents.insert(document.id.clone(), document);
    }
    
    /// 根据向量相似度搜索文档
    pub fn search_by_vector(&self, query_vector: &[f32], top_k: usize) -> Vec<(Document, f32)> {
        let mut results: Vec<(Document, f32)> = Vec::new();
        
        // 创建一个查询文档
        let query_doc = Document {
            id: "query".to_string(),
            content: "".to_string(),
            metadata: HashMap::new(),
            embedding: Some(query_vector.to_vec()),
        };
        
        // 计算所有文档的相似度
        for doc in self.documents.values() {
            if let Some(similarity) = query_doc.cosine_similarity(doc) {
                results.push((doc.clone(), similarity));
            }
        }
        
        // 排序并取前K个
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        
        results
    }
    
    /// 根据文本搜索文档（简单实现）
    pub fn search_by_text(&self, text: &str, top_k: usize) -> Vec<Document> {
        let mut results: Vec<(Document, usize)> = Vec::new();
        
        for doc in self.documents.values() {
            // 简单计数文本中单词在文档中出现的次数
            let count = text.split_whitespace()
                .filter(|word| doc.content.contains(word))
                .count();
            
            if count > 0 {
                results.push((doc.clone(), count));
            }
        }
        
        // 按相关性排序
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(top_k);
        
        results.into_iter().map(|(doc, _)| doc).collect()
    }
    
    /// 获取所有文档
    pub fn get_all_documents(&self) -> Vec<Document> {
        self.documents.values().cloned().collect()
    }
    
    /// 根据ID获取文档
    pub fn get_document(&self, id: &str) -> Option<Document> {
        self.documents.get(id).cloned()
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