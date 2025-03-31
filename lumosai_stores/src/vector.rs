use std::collections::HashMap;
use std::fmt::Debug;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::StoreError;

/// Result of a vector query operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Unique ID of the vector
    pub id: String,
    
    /// Similarity score (higher is more similar)
    pub score: f32,
    
    /// Associated metadata with the vector
    pub metadata: HashMap<String, Value>,
    
    /// The vector itself (only included if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
}

/// Statistics about a vector index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Number of vectors in the index
    pub count: usize,
    
    /// Dimension of vectors in the index
    pub dimension: usize,
    
    /// Distance metric used by the index
    pub metric: String,
}

/// Parameters for creating a vector index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIndexParams {
    /// Name of the index to create
    pub index_name: String,
    
    /// Dimension of vectors in the index
    pub dimension: usize,
    
    /// Distance metric to use (e.g., "cosine", "euclidean", "dotproduct")
    #[serde(default = "default_metric")]
    pub metric: String,
}

fn default_metric() -> String {
    "cosine".to_string()
}

/// Parameters for inserting vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertParams {
    /// Name of the index
    pub index_name: String,
    
    /// Vectors to insert
    pub vectors: Vec<Vec<f32>>,
    
    /// Optional metadata for each vector
    #[serde(default)]
    pub metadata: Vec<HashMap<String, Value>>,
    
    /// Optional IDs for each vector (auto-generated if not provided)
    #[serde(default)]
    pub ids: Option<Vec<String>>,
}

/// Parameters for querying vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    /// Name of the index
    pub index_name: String,
    
    /// Vector to use for the query
    pub query_vector: Vec<f32>,
    
    /// Number of results to return
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    
    /// Optional filter to apply
    #[serde(default)]
    pub filter: Option<VectorFilter>,
    
    /// Whether to include the vector in the results
    #[serde(default)]
    pub include_vector: bool,
}

fn default_top_k() -> usize {
    10
}

/// Parameter types for the VectorStore trait
#[derive(Debug, Clone)]
pub enum VectorStoreParams {
    Create(CreateIndexParams),
    Upsert(UpsertParams),
    Query(QueryParams),
}

/// Filter conditions for vector queries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VectorFilter {
    /// Logical AND operation
    And {
        #[serde(rename = "$and")]
        and: Vec<VectorFilter>,
    },
    
    /// Logical OR operation
    Or {
        #[serde(rename = "$or")]
        or: Vec<VectorFilter>,
    },
    
    /// Logical NOT operation
    Not {
        #[serde(rename = "$not")]
        not: Box<VectorFilter>,
    },
    
    /// Field comparison
    Field(HashMap<String, FieldCondition>),
}

/// Field-specific condition for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldCondition {
    /// Direct value comparison (equals)
    Value(Value),
    
    /// Operator-based comparison
    Operator(HashMap<String, Value>),
}

/// Base filter translator trait for implementing provider-specific translations
#[async_trait]
pub trait VectorFilterTranslator: Send + Sync {
    /// Translate a generic VectorFilter to a provider-specific format
    async fn translate_filter<T: Send + Sync>(&self, filter: Option<VectorFilter>) -> Result<T, StoreError>;
}

/// Common interface for vector stores
#[async_trait]
pub trait VectorStore: Send + Sync + Debug {
    /// Create a new vector index
    async fn create_index(&self, params: CreateIndexParams) -> Result<(), StoreError>;
    
    /// Insert or update vectors in an index
    async fn upsert(&self, params: UpsertParams) -> Result<Vec<String>, StoreError>;
    
    /// Query vectors in an index
    async fn query(&self, params: QueryParams) -> Result<Vec<QueryResult>, StoreError>;
    
    /// List all indexes
    async fn list_indexes(&self) -> Result<Vec<String>, StoreError>;
    
    /// Get statistics about an index
    async fn describe_index(&self, index_name: &str) -> Result<IndexStats, StoreError>;
    
    /// Delete an index
    async fn delete_index(&self, index_name: &str) -> Result<(), StoreError>;
    
    /// Update a vector by ID
    async fn update_vector_by_id(&self, 
        index_name: &str, 
        id: &str, 
        vector: Option<Vec<f32>>, 
        metadata: Option<HashMap<String, Value>>
    ) -> Result<(), StoreError>;
    
    /// Delete vectors by ID
    async fn delete_vectors(&self, index_name: &str, ids: &[String]) -> Result<(), StoreError>;
} 