//! Core types for the Lumos vector storage system

use std::collections::HashMap;
use uuid::Uuid;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// Add chrono dependency for timestamps
#[cfg(feature = "serde")]
use chrono;

/// Vector type alias for f32 vectors
pub type Vector = Vec<f32>;

/// Document ID type
pub type DocumentId = String;

/// Metadata type for storing arbitrary key-value pairs
pub type Metadata = HashMap<String, MetadataValue>;

/// Metadata value that can hold various types
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<MetadataValue>),
    Object(HashMap<String, MetadataValue>),
    Null,
}

impl From<String> for MetadataValue {
    fn from(s: String) -> Self {
        MetadataValue::String(s)
    }
}

impl From<&str> for MetadataValue {
    fn from(s: &str) -> Self {
        MetadataValue::String(s.to_string())
    }
}

impl From<i64> for MetadataValue {
    fn from(i: i64) -> Self {
        MetadataValue::Integer(i)
    }
}

impl From<f64> for MetadataValue {
    fn from(f: f64) -> Self {
        MetadataValue::Float(f)
    }
}

impl From<bool> for MetadataValue {
    fn from(b: bool) -> Self {
        MetadataValue::Boolean(b)
    }
}

/// Similarity metrics for vector comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SimilarityMetric {
    /// Cosine similarity (normalized dot product)
    Cosine,
    /// Euclidean distance (L2 norm)
    Euclidean,
    /// Dot product similarity
    DotProduct,
    /// Manhattan distance (L1 norm)
    Manhattan,
    /// Hamming distance (for binary vectors)
    Hamming,
}

impl Default for SimilarityMetric {
    fn default() -> Self {
        SimilarityMetric::Cosine
    }
}

/// Filter conditions for querying vectors
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FilterCondition {
    /// Equality filter: field == value
    Eq(String, MetadataValue),
    /// Not equal filter: field != value
    Ne(String, MetadataValue),
    /// Greater than filter: field > value
    Gt(String, MetadataValue),
    /// Greater than or equal filter: field >= value
    Gte(String, MetadataValue),
    /// Less than filter: field < value
    Lt(String, MetadataValue),
    /// Less than or equal filter: field <= value
    Lte(String, MetadataValue),
    /// In filter: field in [values]
    In(String, Vec<MetadataValue>),
    /// Not in filter: field not in [values]
    NotIn(String, Vec<MetadataValue>),
    /// Exists filter: field exists
    Exists(String),
    /// Not exists filter: field does not exist
    NotExists(String),
    /// Text contains filter: field contains substring
    Contains(String, String),
    /// Text starts with filter: field starts with prefix
    StartsWith(String, String),
    /// Text ends with filter: field ends with suffix
    EndsWith(String, String),
    /// Regex match filter: field matches regex
    Regex(String, String),
    /// Logical AND: all conditions must be true
    And(Vec<FilterCondition>),
    /// Logical OR: at least one condition must be true
    Or(Vec<FilterCondition>),
    /// Logical NOT: condition must be false
    Not(Box<FilterCondition>),
}

impl FilterCondition {
    /// Create an equality filter
    pub fn eq(field: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        FilterCondition::Eq(field.into(), value.into())
    }
    
    /// Create a not equal filter
    pub fn ne(field: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        FilterCondition::Ne(field.into(), value.into())
    }
    
    /// Create a greater than filter
    pub fn gt(field: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        FilterCondition::Gt(field.into(), value.into())
    }
    
    /// Create a less than filter
    pub fn lt(field: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        FilterCondition::Lt(field.into(), value.into())
    }
    
    /// Create an in filter
    pub fn in_values(field: impl Into<String>, values: Vec<impl Into<MetadataValue>>) -> Self {
        FilterCondition::In(
            field.into(),
            values.into_iter().map(|v| v.into()).collect(),
        )
    }
    
    /// Create an AND filter
    pub fn and(conditions: Vec<FilterCondition>) -> Self {
        FilterCondition::And(conditions)
    }
    
    /// Create an OR filter
    pub fn or(conditions: Vec<FilterCondition>) -> Self {
        FilterCondition::Or(conditions)
    }
    
    /// Create a NOT filter
    pub fn not(condition: FilterCondition) -> Self {
        FilterCondition::Not(Box::new(condition))
    }
    
    /// Create an exists filter
    pub fn exists(field: impl Into<String>) -> Self {
        FilterCondition::Exists(field.into())
    }
    
    /// Create a contains filter
    pub fn contains(field: impl Into<String>, substring: impl Into<String>) -> Self {
        FilterCondition::Contains(field.into(), substring.into())
    }
}

/// Index configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndexConfig {
    /// Index name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric
    pub metric: SimilarityMetric,
    /// Optional index-specific configuration
    pub options: HashMap<String, MetadataValue>,
}

impl IndexConfig {
    /// Create a new index configuration
    pub fn new(name: impl Into<String>, dimension: usize) -> Self {
        Self {
            name: name.into(),
            dimension,
            metric: SimilarityMetric::default(),
            options: HashMap::new(),
        }
    }
    
    /// Set the similarity metric
    pub fn with_metric(mut self, metric: SimilarityMetric) -> Self {
        self.metric = metric;
        self
    }
    
    /// Add an option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        self.options.insert(key.into(), value.into());
        self
    }
}

/// Index statistics and information
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndexInfo {
    /// Index name
    pub name: String,
    /// Vector dimension
    pub dimension: usize,
    /// Similarity metric
    pub metric: SimilarityMetric,
    /// Number of vectors in the index
    pub vector_count: usize,
    /// Index size in bytes
    pub size_bytes: u64,
    /// Index creation timestamp
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Index last updated timestamp
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional index metadata
    pub metadata: Metadata,
}

/// Document representation with embedding support
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Document {
    /// Document ID
    pub id: DocumentId,
    /// Document content/text
    pub content: String,
    /// Vector embedding (optional)
    pub embedding: Option<Vector>,
    /// Document metadata
    pub metadata: Metadata,
}

impl Document {
    /// Create a new document
    pub fn new(id: impl Into<DocumentId>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            embedding: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create a new document with auto-generated ID
    pub fn with_content(content: impl Into<String>) -> Self {
        Self::new(Uuid::new_v4().to_string(), content)
    }
    
    /// Set the embedding
    pub fn with_embedding(mut self, embedding: Vector) -> Self {
        self.embedding = Some(embedding);
        self
    }
    
    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<MetadataValue>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Set all metadata
    pub fn with_all_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Search request for querying vectors
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchRequest {
    /// Index name to search
    pub index_name: String,
    /// Query vector or text
    pub query: SearchQuery,
    /// Number of results to return
    pub top_k: usize,
    /// Optional filter conditions
    pub filter: Option<FilterCondition>,
    /// Whether to include vectors in results
    pub include_vectors: bool,
    /// Whether to include metadata in results
    pub include_metadata: bool,
    /// Search options
    pub options: HashMap<String, MetadataValue>,
}

impl SearchRequest {
    /// Create a new search request with a vector query
    pub fn new(index_name: impl Into<String>, vector: Vector) -> Self {
        Self {
            index_name: index_name.into(),
            query: SearchQuery::Vector(vector),
            top_k: 10,
            filter: None,
            include_vectors: false,
            include_metadata: true,
            options: HashMap::new(),
        }
    }

    /// Create a new search request with a text query
    pub fn new_text(index_name: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            index_name: index_name.into(),
            query: SearchQuery::Text(text.into()),
            top_k: 10,
            filter: None,
            include_vectors: false,
            include_metadata: true,
            options: HashMap::new(),
        }
    }

    /// Set the number of results to return
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Set the filter condition
    pub fn with_filter(mut self, filter: FilterCondition) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set whether to include vectors in results
    pub fn with_include_vectors(mut self, include: bool) -> Self {
        self.include_vectors = include;
        self
    }

    /// Set whether to include metadata in results
    pub fn with_include_metadata(mut self, include: bool) -> Self {
        self.include_metadata = include;
        self
    }

    /// Add a search option
    pub fn with_option(mut self, key: impl Into<String>, value: MetadataValue) -> Self {
        self.options.insert(key.into(), value);
        self
    }
}

/// Search query can be either a vector or text
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SearchQuery {
    /// Vector query
    Vector(Vector),
    /// Text query (requires embedding model)
    Text(String),
}

/// Search result item
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchResult {
    /// Document ID
    pub id: DocumentId,
    /// Similarity score
    pub score: f32,
    /// Document vector (if requested)
    pub vector: Option<Vector>,
    /// Document metadata (if requested)
    pub metadata: Option<Metadata>,
    /// Document content (if available)
    pub content: Option<String>,
}

impl SearchResult {
    /// Create a new search result
    pub fn new(id: impl Into<DocumentId>, score: f32) -> Self {
        Self {
            id: id.into(),
            score,
            vector: None,
            metadata: None,
            content: None,
        }
    }
    
    /// Set the vector
    pub fn with_vector(mut self, vector: Vector) -> Self {
        self.vector = Some(vector);
        self
    }
    
    /// Set the metadata
    pub fn with_metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
    
    /// Set the content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}

/// Search response
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SearchResponse {
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total number of results (before pagination)
    pub total_count: Option<usize>,
    /// Search execution time in milliseconds
    pub execution_time_ms: Option<u64>,
    /// Additional response metadata
    pub metadata: Metadata,
}

impl SearchResponse {
    /// Create a new search response
    pub fn new(results: Vec<SearchResult>) -> Self {
        Self {
            results,
            total_count: None,
            execution_time_ms: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Set the total count
    pub fn with_total_count(mut self, count: usize) -> Self {
        self.total_count = Some(count);
        self
    }
    
    /// Set the execution time
    pub fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = Some(time_ms);
        self
    }
}
