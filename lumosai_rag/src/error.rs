use thiserror::Error;

/// Errors that can occur in the RAG module
#[derive(Error, Debug)]
pub enum RagError {
    /// Error during document loading
    #[error("Document loading error: {0}")]
    DocumentLoading(String),

    /// Error during document parsing
    #[error("Document parsing error: {0}")]
    DocumentParsing(String),

    /// Error during document chunking
    #[error("Document chunking error: {0}")]
    DocumentChunking(String),

    /// Error generating embeddings
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Error with vector store operations
    #[error("Vector store error: {0}")]
    VectorStore(String),

    /// Error during retrieval
    #[error("Retrieval error: {0}")]
    Retrieval(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Configuration(String),

    /// Wrapped error from lumosai_core
    #[error("Core error: {0}")]
    Core(#[from] lumosai_core::error::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for RAG operations
pub type Result<T> = std::result::Result<T, RagError>; 