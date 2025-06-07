//! Weaviate-specific error types

use thiserror::Error;
use lumosai_vector_core::prelude::VectorError;

/// Weaviate-specific error type
#[derive(Error, Debug)]
pub enum WeaviateError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("URL parsing failed: {0}")]
    Url(#[from] url::ParseError),
    
    #[error("Weaviate API error: {0}")]
    Api(String),
    
    #[error("Schema error: {0}")]
    Schema(String),
    
    #[error("Class not found: {0}")]
    ClassNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
}

/// Result type for Weaviate operations
pub type WeaviateResult<T> = Result<T, WeaviateError>;

impl From<WeaviateError> for VectorError {
    fn from(err: WeaviateError) -> Self {
        match err {
            WeaviateError::Http(e) => VectorError::ConnectionFailed(e.to_string()),
            WeaviateError::Json(e) => VectorError::Serialization(e.to_string()),
            WeaviateError::Url(e) => VectorError::InvalidConfig(e.to_string()),
            WeaviateError::Api(msg) => VectorError::OperationFailed(msg),
            WeaviateError::Schema(msg) => VectorError::InvalidIndexConfig(msg),
            WeaviateError::ClassNotFound(msg) => VectorError::IndexNotFound(msg),
            WeaviateError::Config(msg) => VectorError::InvalidConfig(msg),
        }
    }
}
