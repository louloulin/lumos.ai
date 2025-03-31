use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

/// Vector index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Number of dimensions
    pub dimension: usize,
    /// Similarity metric to use
    pub metric: SimilarityMetric,
    /// Optional index parameters
    pub params: Option<HashMap<String, serde_json::Value>>,
}

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

/// Vector with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    /// Vector ID
    pub id: String,
    /// Vector data
    pub data: Vec<f32>,
    /// Associated metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Embedding service for text to vector conversion
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generate embeddings for a list of texts
    async fn embed_texts(&self, texts: &[String]) -> crate::error::Result<Vec<Vec<f32>>>;
    
    /// Get the dimension of the generated embeddings
    fn embedding_dimension(&self) -> usize;
    
    /// Get the model name
    fn model_name(&self) -> &str;
}

/// Vector search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    /// Query vector
    pub query: Vec<f32>,
    /// Number of results to return
    pub top_k: usize,
    /// Filter condition
    pub filter: Option<FilterCondition>,
    /// Whether to include vectors in results
    pub include_vectors: bool,
    /// Minimum similarity score threshold
    pub min_score: Option<f32>,
}

/// Filter condition for vector queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    /// Equals condition
    Eq(String, serde_json::Value),
    /// Greater than condition
    Gt(String, serde_json::Value),
    /// Less than condition
    Lt(String, serde_json::Value),
    /// In array condition
    In(String, Vec<serde_json::Value>),
    /// And condition
    And(Vec<FilterCondition>),
    /// Or condition
    Or(Vec<FilterCondition>),
    /// Not condition
    Not(Box<FilterCondition>),
} 