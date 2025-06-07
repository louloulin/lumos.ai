//! Error types for Qdrant vector storage

use lumosai_vector_core::VectorError;
use thiserror::Error;

/// Qdrant-specific error types
#[derive(Error, Debug)]
pub enum QdrantError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Collection error
    #[error("Collection error: {0}")]
    Collection(String),
    
    /// Point operation error
    #[error("Point operation error: {0}")]
    Point(String),
    
    /// Search error
    #[error("Search error: {0}")]
    Search(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Qdrant client error
    #[error("Qdrant client error: {0}")]
    Client(#[from] qdrant_client::QdrantError),
    
    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// UUID error
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
}

impl From<QdrantError> for VectorError {
    fn from(error: QdrantError) -> Self {
        match error {
            QdrantError::Connection(msg) => VectorError::connection_failed(msg),
            QdrantError::Collection(msg) => VectorError::index_not_found(msg),
            QdrantError::Point(msg) => VectorError::storage_backend(msg),
            QdrantError::Search(msg) => VectorError::search_failed(msg),
            QdrantError::Config(msg) => VectorError::invalid_config(msg),
            QdrantError::Serialization(msg) => VectorError::serialization_failed(msg),
            QdrantError::Client(e) => VectorError::storage_backend(e.to_string()),
            QdrantError::Json(e) => VectorError::serialization_failed(e.to_string()),
            QdrantError::Uuid(e) => VectorError::invalid_input(e.to_string()),
        }
    }
}

/// Result type for Qdrant operations
pub type QdrantResult<T> = Result<T, QdrantError>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_conversion() {
        let qdrant_error = QdrantError::Connection("test error".to_string());
        let vector_error: VectorError = qdrant_error.into();
        
        match vector_error {
            VectorError::ConnectionFailed { .. } => (),
            _ => panic!("Expected ConnectionFailed error"),
        }
    }
    
    #[test]
    fn test_error_display() {
        let error = QdrantError::Collection("test collection error".to_string());
        assert_eq!(error.to_string(), "Collection error: test collection error");
    }
}
