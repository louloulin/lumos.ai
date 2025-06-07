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

/// Chunking strategy for document processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkingStrategy {
    /// Recursive character-based chunking (default)
    Recursive {
        separators: Option<Vec<String>>,
        is_separator_regex: bool,
    },
    /// Simple character-based chunking
    Character {
        separator: String,
        is_separator_regex: bool,
    },
    /// Token-based chunking
    Token {
        encoding_name: Option<String>,
        model_name: Option<String>,
    },
    /// Markdown-aware chunking
    Markdown {
        headers: Option<Vec<String>>,
        return_each_line: bool,
        strip_headers: bool,
    },
    /// HTML-aware chunking
    Html {
        headers: Option<Vec<String>>,
        sections: Option<Vec<String>>,
        return_each_line: bool,
    },
    /// JSON-aware chunking
    Json {
        ensure_ascii: bool,
        convert_lists: bool,
    },
    /// LaTeX-aware chunking
    Latex,
}

impl Default for ChunkingStrategy {
    fn default() -> Self {
        Self::Recursive {
            separators: None,
            is_separator_regex: false,
        }
    }
}

/// Configuration for document chunking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Size of each chunk in tokens or characters
    pub chunk_size: usize,

    /// Overlap between consecutive chunks
    pub chunk_overlap: usize,

    /// Minimum chunk size
    pub min_chunk_size: Option<usize>,

    /// Maximum chunk size
    pub max_chunk_size: Option<usize>,

    /// Chunking strategy to use
    pub strategy: ChunkingStrategy,

    /// Whether to preserve metadata across chunks
    pub preserve_metadata: bool,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            chunk_overlap: 200,
            min_chunk_size: Some(100),
            max_chunk_size: Some(2000),
            strategy: ChunkingStrategy::default(),
            preserve_metadata: true,
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
    /// The documents retrieved with scores
    pub documents: Vec<ScoredDocument>,

    /// Total number of documents found (before limit)
    pub total_count: usize,
}

/// Document with similarity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredDocument {
    /// The document
    pub document: Document,

    /// Similarity score (0.0 to 1.0)
    pub score: f32,
}

/// Request for document retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRequest {
    /// Query text or vector
    pub query: String,

    /// Retrieval options
    pub options: RetrievalOptions,
}

/// Options for document retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalOptions {
    /// Number of documents to retrieve
    pub limit: Option<usize>,

    /// Minimum similarity score threshold
    pub threshold: Option<f32>,

    /// Filter to apply on document metadata
    pub filter: Option<HashMap<String, serde_json::Value>>,
}

impl Default for RetrievalOptions {
    fn default() -> Self {
        Self {
            limit: Some(5),
            threshold: None,
            filter: None,
        }
    }
}

/// Document type for different content formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Text,
    Markdown,
    Html,
    Json,
    Latex,
    Pdf,
    Docx,
    Csv,
}

impl Default for DocumentType {
    fn default() -> Self {
        Self::Text
    }
}

/// Metadata extraction configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractionConfig {
    /// Extract title from document
    pub extract_title: bool,

    /// Extract summary from document
    pub extract_summary: bool,

    /// Extract keywords from document
    pub extract_keywords: bool,

    /// Extract questions answered by document
    pub extract_questions: bool,

    /// Custom extraction prompts
    pub custom_extractions: HashMap<String, String>,
}

/// Node relationship types for document hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeRelationship {
    Source,
    Previous,
    Next,
    Parent,
    Child,
}

/// Document processing pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Document type
    pub document_type: DocumentType,

    /// Chunking configuration
    pub chunking: ChunkingConfig,

    /// Embedding configuration
    pub embedding: EmbeddingConfig,

    /// Metadata extraction configuration
    pub extraction: ExtractionConfig,

    /// Whether to clean text (remove extra whitespace, etc.)
    pub clean_text: bool,

    /// Whether to normalize unicode
    pub normalize_unicode: bool,
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            document_type: DocumentType::default(),
            chunking: ChunkingConfig::default(),
            embedding: EmbeddingConfig {
                provider: "openai".to_string(),
                model: "text-embedding-3-small".to_string(),
                dimensions: 1536,
                extra_config: HashMap::new(),
            },
            extraction: ExtractionConfig::default(),
            clean_text: true,
            normalize_unicode: true,
        }
    }
}