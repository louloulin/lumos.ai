//! Error types for LanceDB integration

use thiserror::Error;

/// Result type for LanceDB operations
pub type LanceDbResult<T> = std::result::Result<T, LanceDbError>;

/// Errors that can occur when using LanceDB
#[derive(Error, Debug)]
pub enum LanceDbError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Database operation error
    #[error("Database error: {0}")]
    Database(String),
    
    /// Table operation error
    #[error("Table error: {0}")]
    Table(String),
    
    /// Index operation error
    #[error("Index error: {0}")]
    Index(String),
    
    /// Query error
    #[error("Query error: {0}")]
    Query(String),
    
    /// Invalid data error
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    /// Invalid configuration error
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Arrow error
    #[error("Arrow error: {0}")]
    Arrow(String),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Timeout error
    #[error("Operation timed out")]
    Timeout,
    
    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Already exists error
    #[error("Already exists: {0}")]
    AlreadyExists(String),
    
    /// Permission denied error
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    /// Storage error (cloud storage related)
    #[error("Storage error: {0}")]
    Storage(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Generic error
    #[error("LanceDB error: {0}")]
    Generic(String),
}

impl LanceDbError {
    /// Create a new connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Self::Connection(msg.into())
    }
    
    /// Create a new database error
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::Database(msg.into())
    }
    
    /// Create a new table error
    pub fn table<S: Into<String>>(msg: S) -> Self {
        Self::Table(msg.into())
    }
    
    /// Create a new index error
    pub fn index<S: Into<String>>(msg: S) -> Self {
        Self::Index(msg.into())
    }
    
    /// Create a new query error
    pub fn query<S: Into<String>>(msg: S) -> Self {
        Self::Query(msg.into())
    }
    
    /// Create a new invalid data error
    pub fn invalid_data<S: Into<String>>(msg: S) -> Self {
        Self::InvalidData(msg.into())
    }
    
    /// Create a new configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::InvalidConfiguration(msg.into())
    }
    
    /// Create a new serialization error
    pub fn serialization<S: Into<String>>(msg: S) -> Self {
        Self::Serialization(msg.into())
    }
    
    /// Create a new arrow error
    pub fn arrow<S: Into<String>>(msg: S) -> Self {
        Self::Arrow(msg.into())
    }
    
    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }
    
    /// Create a new already exists error
    pub fn already_exists<S: Into<String>>(msg: S) -> Self {
        Self::AlreadyExists(msg.into())
    }
    
    /// Create a new storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Self::Storage(msg.into())
    }
    
    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        Self::Authentication(msg.into())
    }
    
    /// Create a new generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            LanceDbError::Connection(_) => true,
            LanceDbError::Database(_) => true,
            LanceDbError::Table(_) => true,
            LanceDbError::Index(_) => true,
            LanceDbError::Query(_) => true,
            LanceDbError::InvalidData(_) => false,
            LanceDbError::InvalidConfiguration(_) => false,
            LanceDbError::Serialization(_) => false,
            LanceDbError::Arrow(_) => false,
            LanceDbError::Io(_) => true,
            LanceDbError::Timeout => true,
            LanceDbError::NotFound(_) => false,
            LanceDbError::AlreadyExists(_) => false,
            LanceDbError::PermissionDenied(_) => false,
            LanceDbError::Storage(_) => true,
            LanceDbError::Authentication(_) => false,
            LanceDbError::Generic(_) => true,
        }
    }
    
    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            LanceDbError::Connection(_) => "connection",
            LanceDbError::Database(_) => "database",
            LanceDbError::Table(_) => "table",
            LanceDbError::Index(_) => "index",
            LanceDbError::Query(_) => "query",
            LanceDbError::InvalidData(_) => "invalid_data",
            LanceDbError::InvalidConfiguration(_) => "config",
            LanceDbError::Serialization(_) => "serialization",
            LanceDbError::Arrow(_) => "arrow",
            LanceDbError::Io(_) => "io",
            LanceDbError::Timeout => "timeout",
            LanceDbError::NotFound(_) => "not_found",
            LanceDbError::AlreadyExists(_) => "already_exists",
            LanceDbError::PermissionDenied(_) => "permission_denied",
            LanceDbError::Storage(_) => "storage",
            LanceDbError::Authentication(_) => "authentication",
            LanceDbError::Generic(_) => "generic",
        }
    }
    
    /// Check if this is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        match self {
            LanceDbError::InvalidData(_) => true,
            LanceDbError::InvalidConfiguration(_) => true,
            LanceDbError::NotFound(_) => true,
            LanceDbError::AlreadyExists(_) => true,
            LanceDbError::PermissionDenied(_) => true,
            LanceDbError::Authentication(_) => true,
            _ => false,
        }
    }
    
    /// Check if this is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        match self {
            LanceDbError::Connection(_) => true,
            LanceDbError::Database(_) => true,
            LanceDbError::Table(_) => true,
            LanceDbError::Index(_) => true,
            LanceDbError::Storage(_) => true,
            LanceDbError::Timeout => true,
            _ => false,
        }
    }
}

// Conversion from LanceDB errors
impl From<lancedb::Error> for LanceDbError {
    fn from(err: lancedb::Error) -> Self {
        // Map LanceDB errors to our error types
        let error_str = err.to_string();
        
        if error_str.contains("connection") || error_str.contains("connect") {
            LanceDbError::Connection(error_str)
        } else if error_str.contains("table") {
            LanceDbError::Table(error_str)
        } else if error_str.contains("index") {
            LanceDbError::Index(error_str)
        } else if error_str.contains("query") {
            LanceDbError::Query(error_str)
        } else if error_str.contains("not found") {
            LanceDbError::NotFound(error_str)
        } else if error_str.contains("already exists") {
            LanceDbError::AlreadyExists(error_str)
        } else if error_str.contains("timeout") {
            LanceDbError::Timeout
        } else if error_str.contains("permission") || error_str.contains("access") {
            LanceDbError::PermissionDenied(error_str)
        } else if error_str.contains("auth") {
            LanceDbError::Authentication(error_str)
        } else {
            LanceDbError::Database(error_str)
        }
    }
}

// Conversion from Arrow errors
impl From<arrow::error::ArrowError> for LanceDbError {
    fn from(err: arrow::error::ArrowError) -> Self {
        LanceDbError::Arrow(err.to_string())
    }
}

// Conversion from serde_json errors
impl From<serde_json::Error> for LanceDbError {
    fn from(err: serde_json::Error) -> Self {
        LanceDbError::Serialization(err.to_string())
    }
}

// Conversion to lumosai_vector_core::error::VectorError
impl From<LanceDbError> for lumosai_vector_core::error::VectorError {
    fn from(err: LanceDbError) -> Self {
        match err {
            LanceDbError::Connection(msg) => lumosai_vector_core::error::VectorError::ConnectionError(msg),
            LanceDbError::Database(msg) => lumosai_vector_core::error::VectorError::DatabaseError(msg),
            LanceDbError::Table(msg) => lumosai_vector_core::error::VectorError::DatabaseError(msg),
            LanceDbError::Index(msg) => lumosai_vector_core::error::VectorError::IndexError(msg),
            LanceDbError::Query(msg) => lumosai_vector_core::error::VectorError::QueryError(msg),
            LanceDbError::InvalidData(msg) => lumosai_vector_core::error::VectorError::InvalidVector(msg),
            LanceDbError::InvalidConfiguration(msg) => lumosai_vector_core::error::VectorError::ConfigurationError(msg),
            LanceDbError::Serialization(msg) => lumosai_vector_core::error::VectorError::SerializationError(msg),
            LanceDbError::Arrow(msg) => lumosai_vector_core::error::VectorError::Internal(msg),
            LanceDbError::Io(err) => lumosai_vector_core::error::VectorError::Io(err),
            LanceDbError::Timeout => lumosai_vector_core::error::VectorError::Timeout,
            LanceDbError::NotFound(msg) => lumosai_vector_core::error::VectorError::NotFound(msg),
            LanceDbError::AlreadyExists(msg) => lumosai_vector_core::error::VectorError::AlreadyExists(msg),
            LanceDbError::PermissionDenied(msg) => lumosai_vector_core::error::VectorError::PermissionDenied(msg),
            LanceDbError::Storage(msg) => lumosai_vector_core::error::VectorError::Internal(msg),
            LanceDbError::Authentication(msg) => lumosai_vector_core::error::VectorError::AuthenticationError(msg),
            LanceDbError::Generic(msg) => lumosai_vector_core::error::VectorError::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = LanceDbError::connection("Test connection error");
        assert!(matches!(err, LanceDbError::Connection(_)));
        assert!(err.is_recoverable());
        assert!(err.is_server_error());
        assert!(!err.is_client_error());
        assert_eq!(err.category(), "connection");
    }
    
    #[test]
    fn test_error_categories() {
        let client_err = LanceDbError::not_found("Table not found");
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());
        assert!(!client_err.is_recoverable());
        
        let server_err = LanceDbError::timeout();
        assert!(server_err.is_server_error());
        assert!(!server_err.is_client_error());
        assert!(server_err.is_recoverable());
    }
    
    #[test]
    fn test_error_conversion() {
        let lancedb_err = LanceDbError::database("Test database error");
        let vector_err: lumosai_vector_core::error::VectorError = lancedb_err.into();
        
        match vector_err {
            lumosai_vector_core::error::VectorError::DatabaseError(msg) => {
                assert!(msg.contains("Test database error"));
            }
            _ => panic!("Expected DatabaseError"),
        }
    }
    
    #[test]
    fn test_timeout_error() {
        let err = LanceDbError::Timeout;
        assert_eq!(err.category(), "timeout");
        assert!(err.is_recoverable());
        assert!(err.is_server_error());
    }
}
