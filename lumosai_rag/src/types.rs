use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a document or a chunk of a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier for the document
    pub id: String,
    
    /// The document content
    pub content: String,
    
    /// The document metadata
    pub metadata: Metadata,
    
    /// The embedding vector if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

/// Metadata attached to a document
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    /// Arbitrary metadata fields
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
    
    /// Optional source information (e.g., file path, URL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    
    /// Optional creation date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Metadata {
    /// Create a new empty metadata object
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a field to the metadata
    pub fn add<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<serde_json::Value>,
    {
        self.fields.insert(key.into(), value.into());
        self
    }
    
    /// Set the source of the document
    pub fn with_source<S: Into<String>>(mut self, source: S) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// Configuration for document chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Size of each chunk in tokens or characters
    pub chunk_size: usize,
    
    /// Overlap between consecutive chunks
    pub chunk_overlap: usize,
    
    /// Whether to chunk by tokens (true) or characters/bytes (false)
    pub chunk_by_tokens: bool,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 200,
            chunk_by_tokens: false,
        }
    }
}

/// Configuration for embedding generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// The provider for embeddings (e.g., OpenAI, local)
    pub provider: String,
    
    /// The model name
    pub model: String,
    
    /// Dimension of the embedding vectors
    pub dimensions: usize,
    
    /// Additional provider-specific configuration
    #[serde(flatten)]
    pub extra_config: HashMap<String, serde_json::Value>,
}

/// Result from a document retrieval operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// The documents retrieved
    pub documents: Vec<Document>,
    
    /// Optional similarity scores for each document
    pub scores: Option<Vec<f32>>,
}

/// Options for document retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalOptions {
    /// Number of documents to retrieve
    pub limit: usize,
    
    /// Minimum similarity score threshold
    pub threshold: Option<f32>,
    
    /// Filter to apply on document metadata
    pub filter: Option<HashMap<String, serde_json::Value>>,
}

impl Default for RetrievalOptions {
    fn default() -> Self {
        Self {
            limit: 5,
            threshold: None,
            filter: None,
        }
    }
} 