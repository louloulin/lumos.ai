//! Core types for vector storage operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector type alias
pub type Vector = Vec<f32>;

/// Similarity metrics for vector comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SimilarityMetric {
    /// Cosine similarity (normalized dot product)
    Cosine,
    /// Euclidean distance (L2 norm)
    Euclidean,
    /// Dot product similarity
    DotProduct,
}

impl Default for SimilarityMetric {
    fn default() -> Self {
        Self::Cosine
    }
}

/// Index configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    /// Index name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric
    pub metric: SimilarityMetric,
    /// Additional configuration parameters
    pub params: HashMap<String, serde_json::Value>,
}

impl IndexConfig {
    /// Create a new index configuration
    pub fn new(name: String, dimension: usize, metric: SimilarityMetric) -> Self {
        Self {
            name,
            dimension,
            metric,
            params: HashMap::new(),
        }
    }
    
    /// Add a configuration parameter
    pub fn with_param<K, V>(mut self, key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.params.insert(key.into(), value.into());
        self
    }
}

/// Filter conditions for vector queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterCondition {
    /// Equality filter
    Eq(String, serde_json::Value),
    /// Greater than filter
    Gt(String, serde_json::Value),
    /// Less than filter
    Lt(String, serde_json::Value),
    /// In filter (value in list)
    In(String, Vec<serde_json::Value>),
    /// Logical AND
    And(Vec<FilterCondition>),
    /// Logical OR
    Or(Vec<FilterCondition>),
    /// Logical NOT
    Not(Box<FilterCondition>),
}

impl FilterCondition {
    /// Create an equality filter
    pub fn eq<K, V>(key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        Self::Eq(key.into(), value.into())
    }
    
    /// Create a greater than filter
    pub fn gt<K, V>(key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        Self::Gt(key.into(), value.into())
    }
    
    /// Create a less than filter
    pub fn lt<K, V>(key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        Self::Lt(key.into(), value.into())
    }
    
    /// Create an in filter
    pub fn in_list<K, V>(key: K, values: Vec<V>) -> Self 
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        let values = values.into_iter().map(|v| v.into()).collect();
        Self::In(key.into(), values)
    }
    
    /// Create a logical AND filter
    pub fn and(conditions: Vec<FilterCondition>) -> Self {
        Self::And(conditions)
    }
    
    /// Create a logical OR filter
    pub fn or(conditions: Vec<FilterCondition>) -> Self {
        Self::Or(conditions)
    }
    
    /// Create a logical NOT filter
    pub fn not(condition: FilterCondition) -> Self {
        Self::Not(Box::new(condition))
    }
}

/// Search parameters for vector queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    /// Number of results to return
    pub top_k: usize,
    /// Filter conditions
    pub filter: Option<FilterCondition>,
    /// Whether to include vectors in results
    pub include_vectors: bool,
    /// Additional search parameters
    pub params: HashMap<String, serde_json::Value>,
}

impl SearchParams {
    /// Create new search parameters
    pub fn new(top_k: usize) -> Self {
        Self {
            top_k,
            filter: None,
            include_vectors: false,
            params: HashMap::new(),
        }
    }
    
    /// Set filter condition
    pub fn with_filter(mut self, filter: FilterCondition) -> Self {
        self.filter = Some(filter);
        self
    }
    
    /// Include vectors in results
    pub fn include_vectors(mut self) -> Self {
        self.include_vectors = true;
        self
    }
    
    /// Add a search parameter
    pub fn with_param<K, V>(mut self, key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.params.insert(key.into(), value.into());
        self
    }
}

/// Query result containing vector and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Vector ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Vector data (optional)
    pub vector: Option<Vector>,
    /// Associated metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(id: String, score: f32) -> Self {
        Self {
            id,
            score,
            vector: None,
            metadata: None,
        }
    }
    
    /// Set vector data
    pub fn with_vector(mut self, vector: Vector) -> Self {
        self.vector = Some(vector);
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Index name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric
    pub metric: SimilarityMetric,
    /// Number of vectors
    pub vector_count: usize,
    /// Index size in bytes
    pub size_bytes: u64,
    /// Additional statistics
    pub stats: HashMap<String, serde_json::Value>,
}

impl IndexStats {
    /// Create new index statistics
    pub fn new(name: String, dimension: usize, metric: SimilarityMetric) -> Self {
        Self {
            name,
            dimension,
            metric,
            vector_count: 0,
            size_bytes: 0,
            stats: HashMap::new(),
        }
    }
    
    /// Set vector count
    pub fn with_vector_count(mut self, count: usize) -> Self {
        self.vector_count = count;
        self
    }
    
    /// Set size in bytes
    pub fn with_size_bytes(mut self, size: u64) -> Self {
        self.size_bytes = size;
        self
    }
    
    /// Add a statistic
    pub fn with_stat<K, V>(mut self, key: K, value: V) -> Self 
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.stats.insert(key.into(), value.into());
        self
    }
}
