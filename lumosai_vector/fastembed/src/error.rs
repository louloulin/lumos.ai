//! Error types for FastEmbed integration

use thiserror::Error;

/// Result type for FastEmbed operations
pub type Result<T> = std::result::Result<T, FastEmbedError>;

/// Errors that can occur when using FastEmbed
#[derive(Error, Debug)]
pub enum FastEmbedError {
    /// Model initialization failed
    #[error("Model initialization failed: {0}")]
    ModelInitialization(String),
    
    /// Model not initialized
    #[error("Model not initialized: {0}")]
    ModelNotInitialized(String),
    
    /// Embedding generation failed
    #[error("Embedding generation failed: {0}")]
    EmbeddingGeneration(String),
    
    /// Text is too long for the model
    #[error("Text too long: {length} characters, maximum: {max_length}")]
    TextTooLong { length: usize, max_length: usize },
    
    /// Invalid model configuration
    #[error("Invalid model configuration: {0}")]
    InvalidConfiguration(String),
    
    /// IO error (file operations, cache, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    /// Model download failed
    #[error("Model download failed: {0}")]
    ModelDownload(String),
    
    /// Cache directory error
    #[error("Cache directory error: {0}")]
    CacheDirectory(String),
    
    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// Generic error
    #[error("FastEmbed error: {0}")]
    Generic(String),
}

impl FastEmbedError {
    /// Create a new model initialization error
    pub fn model_init<S: Into<String>>(msg: S) -> Self {
        Self::ModelInitialization(msg.into())
    }
    
    /// Create a new embedding generation error
    pub fn embedding<S: Into<String>>(msg: S) -> Self {
        Self::EmbeddingGeneration(msg.into())
    }
    
    /// Create a new configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::InvalidConfiguration(msg.into())
    }
    
    /// Create a new generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            FastEmbedError::ModelInitialization(_) => false,
            FastEmbedError::ModelNotInitialized(_) => true,
            FastEmbedError::EmbeddingGeneration(_) => true,
            FastEmbedError::TextTooLong { .. } => true,
            FastEmbedError::InvalidConfiguration(_) => false,
            FastEmbedError::Io(_) => true,
            FastEmbedError::Serialization(_) => false,
            FastEmbedError::ModelDownload(_) => true,
            FastEmbedError::CacheDirectory(_) => true,
            FastEmbedError::UnsupportedOperation(_) => false,
            FastEmbedError::Generic(_) => true,
        }
    }
    
    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            FastEmbedError::ModelInitialization(_) => "model_init",
            FastEmbedError::ModelNotInitialized(_) => "model_not_init",
            FastEmbedError::EmbeddingGeneration(_) => "embedding",
            FastEmbedError::TextTooLong { .. } => "text_length",
            FastEmbedError::InvalidConfiguration(_) => "config",
            FastEmbedError::Io(_) => "io",
            FastEmbedError::Serialization(_) => "serialization",
            FastEmbedError::ModelDownload(_) => "download",
            FastEmbedError::CacheDirectory(_) => "cache",
            FastEmbedError::UnsupportedOperation(_) => "unsupported",
            FastEmbedError::Generic(_) => "generic",
        }
    }
}

// Conversion to lumosai_vector_core::error::VectorError
impl From<FastEmbedError> for lumosai_vector_core::error::VectorError {
    fn from(err: FastEmbedError) -> Self {
        lumosai_vector_core::error::VectorError::EmbeddingError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = FastEmbedError::model_init("Test error");
        assert!(matches!(err, FastEmbedError::ModelInitialization(_)));
        assert!(!err.is_recoverable());
        assert_eq!(err.category(), "model_init");
    }
    
    #[test]
    fn test_text_too_long_error() {
        let err = FastEmbedError::TextTooLong {
            length: 1000,
            max_length: 512,
        };
        assert!(err.is_recoverable());
        assert_eq!(err.category(), "text_length");
    }
    
    #[test]
    fn test_error_conversion() {
        let fastembed_err = FastEmbedError::embedding("Test embedding error");
        let vector_err: lumosai_vector_core::error::VectorError = fastembed_err.into();
        
        match vector_err {
            lumosai_vector_core::error::VectorError::EmbeddingError(msg) => {
                assert!(msg.contains("Test embedding error"));
            }
            _ => panic!("Expected EmbeddingError"),
        }
    }
}
