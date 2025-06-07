//! Error types for vector storage operations

use thiserror::Error;

/// Result type alias for vector storage operations
pub type Result<T> = std::result::Result<T, VectorError>;

/// Error types for vector storage operations
#[derive(Error, Debug)]
pub enum VectorError {
    /// Index not found
    #[error("Index '{0}' not found")]
    IndexNotFound(String),
    
    /// Index already exists
    #[error("Index '{0}' already exists")]
    IndexAlreadyExists(String),
    
    /// Vector dimension mismatch
    #[error("Vector dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    
    /// Vector not found
    #[error("Vector with ID '{0}' not found")]
    VectorNotFound(String),
    
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Network/connection error
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
    
    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl VectorError {
    /// Create a storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Self::Storage(msg.into())
    }
    
    /// Create a serialization error
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Self::Serialization(msg.into())
    }
    
    /// Create a connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Self::Connection(msg.into())
    }
    
    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }
}

// Conversion from common error types
impl From<serde_json::Error> for VectorError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<uuid::Error> for VectorError {
    fn from(err: uuid::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for VectorError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Storage(err.to_string())
    }
}

#[cfg(feature = "qdrant")]
impl From<qdrant_client::QdrantError> for VectorError {
    fn from(err: qdrant_client::QdrantError) -> Self {
        Self::Connection(err.to_string())
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::error::Error> for VectorError {
    fn from(err: mongodb::error::Error) -> Self {
        Self::Storage(err.to_string())
    }
}
