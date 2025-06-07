//! Error types for the Lumos vector storage system

use thiserror::Error;

/// Result type alias for vector operations
pub type Result<T> = std::result::Result<T, VectorError>;

/// Comprehensive error type for vector storage operations
#[derive(Error, Debug, Clone)]
pub enum VectorError {
    /// Index-related errors
    #[error("Index '{0}' not found")]
    IndexNotFound(String),
    
    #[error("Index '{0}' already exists")]
    IndexAlreadyExists(String),
    
    #[error("Invalid index configuration: {0}")]
    InvalidIndexConfig(String),
    
    /// Vector-related errors
    #[error("Vector '{0}' not found")]
    VectorNotFound(String),
    
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    #[error("Invalid vector data: {0}")]
    InvalidVector(String),
    
    /// Query-related errors
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Query timeout after {seconds} seconds")]
    QueryTimeout { seconds: u64 },
    
    #[error("Invalid filter condition: {0}")]
    InvalidFilter(String),
    
    /// Storage backend errors
    #[error("Storage backend error: {0}")]
    StorageBackend(String),
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    /// Configuration errors
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
    
    /// Resource errors
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Insufficient storage space")]
    InsufficientStorage,
    
    #[error("Memory allocation failed")]
    OutOfMemory,
    
    /// Operation errors
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error("Concurrent modification detected")]
    ConcurrentModification,
    
    /// Internal errors
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl VectorError {
    /// Create an index not found error
    pub fn index_not_found(name: impl Into<String>) -> Self {
        Self::IndexNotFound(name.into())
    }
    
    /// Create an index already exists error
    pub fn index_already_exists(name: impl Into<String>) -> Self {
        Self::IndexAlreadyExists(name.into())
    }
    
    /// Create a dimension mismatch error
    pub fn dimension_mismatch(expected: usize, actual: usize) -> Self {
        Self::DimensionMismatch { expected, actual }
    }
    
    /// Create a vector not found error
    pub fn vector_not_found(id: impl Into<String>) -> Self {
        Self::VectorNotFound(id.into())
    }
    
    /// Create a storage backend error
    pub fn storage_backend(msg: impl Into<String>) -> Self {
        Self::StorageBackend(msg.into())
    }
    
    /// Create a connection failed error
    pub fn connection_failed(msg: impl Into<String>) -> Self {
        Self::ConnectionFailed(msg.into())
    }
    
    /// Create an invalid configuration error
    pub fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }
    
    /// Create a serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }
    
    /// Create an internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
    
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            VectorError::QueryTimeout { .. }
                | VectorError::ConnectionFailed(_)
                | VectorError::InsufficientStorage
                | VectorError::ConcurrentModification
        )
    }
    
    /// Check if this error is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            VectorError::IndexNotFound(_)
                | VectorError::VectorNotFound(_)
                | VectorError::DimensionMismatch { .. }
                | VectorError::InvalidQuery(_)
                | VectorError::InvalidFilter(_)
                | VectorError::InvalidVector(_)
                | VectorError::InvalidConfig(_)
                | VectorError::MissingConfig(_)
                | VectorError::NotSupported(_)
        )
    }
    
    /// Check if this error is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            VectorError::StorageBackend(_)
                | VectorError::Internal(_)
                | VectorError::Unexpected(_)
                | VectorError::OutOfMemory
                | VectorError::OperationFailed(_)
        )
    }
}

/// Convert from standard I/O errors
impl From<std::io::Error> for VectorError {
    fn from(err: std::io::Error) -> Self {
        VectorError::StorageBackend(err.to_string())
    }
}

/// Convert from serde JSON errors
#[cfg(feature = "serde")]
impl From<serde_json::Error> for VectorError {
    fn from(err: serde_json::Error) -> Self {
        VectorError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = VectorError::index_not_found("test_index");
        assert!(matches!(err, VectorError::IndexNotFound(_)));
        assert!(err.is_client_error());
        assert!(!err.is_retryable());
    }
    
    #[test]
    fn test_dimension_mismatch() {
        let err = VectorError::dimension_mismatch(384, 512);
        assert!(matches!(err, VectorError::DimensionMismatch { expected: 384, actual: 512 }));
    }
    
    #[test]
    fn test_error_classification() {
        assert!(VectorError::QueryTimeout { seconds: 30 }.is_retryable());
        assert!(VectorError::InvalidQuery("bad query".to_string()).is_client_error());
        assert!(VectorError::Internal("system error".to_string()).is_server_error());
    }
}
