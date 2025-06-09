//! Error types for Milvus integration

use thiserror::Error;

/// Result type for Milvus operations
pub type MilvusResult<T> = std::result::Result<T, MilvusError>;

/// Errors that can occur when using Milvus
#[derive(Error, Debug)]
pub enum MilvusError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Database operation error
    #[error("Database error: {0}")]
    Database(String),
    
    /// Collection operation error
    #[error("Collection error: {0}")]
    Collection(String),
    
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
    
    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(String),
    
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
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    /// Service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    /// Generic error
    #[error("Milvus error: {0}")]
    Generic(String),
}

impl MilvusError {
    /// Create a new connection error
    pub fn connection<S: Into<String>>(msg: S) -> Self {
        Self::Connection(msg.into())
    }
    
    /// Create a new authentication error
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        Self::Authentication(msg.into())
    }
    
    /// Create a new database error
    pub fn database<S: Into<String>>(msg: S) -> Self {
        Self::Database(msg.into())
    }
    
    /// Create a new collection error
    pub fn collection<S: Into<String>>(msg: S) -> Self {
        Self::Collection(msg.into())
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
    
    /// Create a new HTTP error
    pub fn http<S: Into<String>>(msg: S) -> Self {
        Self::Http(msg.into())
    }
    
    /// Create a new not found error
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }
    
    /// Create a new already exists error
    pub fn already_exists<S: Into<String>>(msg: S) -> Self {
        Self::AlreadyExists(msg.into())
    }
    
    /// Create a new generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }
    
    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        match self {
            MilvusError::Connection(_) => true,
            MilvusError::Authentication(_) => false,
            MilvusError::Database(_) => true,
            MilvusError::Collection(_) => true,
            MilvusError::Index(_) => true,
            MilvusError::Query(_) => true,
            MilvusError::InvalidData(_) => false,
            MilvusError::InvalidConfiguration(_) => false,
            MilvusError::Serialization(_) => false,
            MilvusError::Http(_) => true,
            MilvusError::Io(_) => true,
            MilvusError::Timeout => true,
            MilvusError::NotFound(_) => false,
            MilvusError::AlreadyExists(_) => false,
            MilvusError::PermissionDenied(_) => false,
            MilvusError::RateLimitExceeded(_) => true,
            MilvusError::ServiceUnavailable(_) => true,
            MilvusError::Generic(_) => true,
        }
    }
    
    /// Get error category for logging/monitoring
    pub fn category(&self) -> &'static str {
        match self {
            MilvusError::Connection(_) => "connection",
            MilvusError::Authentication(_) => "authentication",
            MilvusError::Database(_) => "database",
            MilvusError::Collection(_) => "collection",
            MilvusError::Index(_) => "index",
            MilvusError::Query(_) => "query",
            MilvusError::InvalidData(_) => "invalid_data",
            MilvusError::InvalidConfiguration(_) => "config",
            MilvusError::Serialization(_) => "serialization",
            MilvusError::Http(_) => "http",
            MilvusError::Io(_) => "io",
            MilvusError::Timeout => "timeout",
            MilvusError::NotFound(_) => "not_found",
            MilvusError::AlreadyExists(_) => "already_exists",
            MilvusError::PermissionDenied(_) => "permission_denied",
            MilvusError::RateLimitExceeded(_) => "rate_limit",
            MilvusError::ServiceUnavailable(_) => "service_unavailable",
            MilvusError::Generic(_) => "generic",
        }
    }
    
    /// Check if this is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        match self {
            MilvusError::InvalidData(_) => true,
            MilvusError::InvalidConfiguration(_) => true,
            MilvusError::NotFound(_) => true,
            MilvusError::AlreadyExists(_) => true,
            MilvusError::PermissionDenied(_) => true,
            MilvusError::Authentication(_) => true,
            MilvusError::RateLimitExceeded(_) => true,
            _ => false,
        }
    }
    
    /// Check if this is a server error (5xx equivalent)
    pub fn is_server_error(&self) -> bool {
        match self {
            MilvusError::Connection(_) => true,
            MilvusError::Database(_) => true,
            MilvusError::Collection(_) => true,
            MilvusError::Index(_) => true,
            MilvusError::ServiceUnavailable(_) => true,
            MilvusError::Timeout => true,
            _ => false,
        }
    }
    
    /// Get HTTP status code equivalent
    pub fn status_code(&self) -> u16 {
        match self {
            MilvusError::NotFound(_) => 404,
            MilvusError::AlreadyExists(_) => 409,
            MilvusError::PermissionDenied(_) => 403,
            MilvusError::Authentication(_) => 401,
            MilvusError::InvalidData(_) => 400,
            MilvusError::InvalidConfiguration(_) => 400,
            MilvusError::RateLimitExceeded(_) => 429,
            MilvusError::ServiceUnavailable(_) => 503,
            MilvusError::Timeout => 408,
            _ => 500,
        }
    }
}

// Conversion from reqwest errors
impl From<reqwest::Error> for MilvusError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            MilvusError::Timeout
        } else if err.is_connect() {
            MilvusError::Connection(err.to_string())
        } else if err.is_request() {
            MilvusError::Http(err.to_string())
        } else {
            MilvusError::Http(err.to_string())
        }
    }
}

// Conversion from serde_json errors
impl From<serde_json::Error> for MilvusError {
    fn from(err: serde_json::Error) -> Self {
        MilvusError::Serialization(err.to_string())
    }
}

// Conversion to lumosai_vector_core::error::VectorError
impl From<MilvusError> for lumosai_vector_core::error::VectorError {
    fn from(err: MilvusError) -> Self {
        match err {
            MilvusError::Connection(msg) => lumosai_vector_core::error::VectorError::ConnectionFailed(msg),
            MilvusError::Authentication(msg) => lumosai_vector_core::error::VectorError::AuthenticationFailed(msg),
            MilvusError::Database(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
            MilvusError::Collection(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
            MilvusError::Index(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
            MilvusError::Query(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
            MilvusError::InvalidData(msg) => lumosai_vector_core::error::VectorError::InvalidVector(msg),
            MilvusError::InvalidConfiguration(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
            MilvusError::Serialization(msg) => lumosai_vector_core::error::VectorError::serialization(msg),
            MilvusError::Http(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
            MilvusError::Io(err) => lumosai_vector_core::error::VectorError::from(err),
            MilvusError::Timeout => lumosai_vector_core::error::VectorError::OperationFailed("Operation timed out".to_string()),
            MilvusError::NotFound(msg) => lumosai_vector_core::error::VectorError::IndexNotFound(msg),
            MilvusError::AlreadyExists(msg) => lumosai_vector_core::error::VectorError::IndexAlreadyExists(msg),
            MilvusError::PermissionDenied(msg) => lumosai_vector_core::error::VectorError::AuthenticationFailed(msg),
            MilvusError::RateLimitExceeded(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
            MilvusError::ServiceUnavailable(msg) => lumosai_vector_core::error::VectorError::ConnectionFailed(msg),
            MilvusError::Generic(msg) => lumosai_vector_core::error::VectorError::OperationFailed(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = MilvusError::connection("Test connection error");
        assert!(matches!(err, MilvusError::Connection(_)));
        assert!(err.is_recoverable());
        assert!(err.is_server_error());
        assert!(!err.is_client_error());
        assert_eq!(err.category(), "connection");
        assert_eq!(err.status_code(), 500);
    }
    
    #[test]
    fn test_error_categories() {
        let client_err = MilvusError::not_found("Collection not found");
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());
        assert!(!client_err.is_recoverable());
        assert_eq!(client_err.status_code(), 404);
        
        let server_err = MilvusError::Timeout;
        assert!(server_err.is_server_error());
        assert!(!server_err.is_client_error());
        assert!(server_err.is_recoverable());
        assert_eq!(server_err.status_code(), 408);
    }
    
    #[test]
    fn test_error_conversion() {
        let milvus_err = MilvusError::database("Test database error");
        let vector_err: lumosai_vector_core::error::VectorError = milvus_err.into();
        
        match vector_err {
            lumosai_vector_core::error::VectorError::DatabaseError(msg) => {
                assert!(msg.contains("Test database error"));
            }
            _ => panic!("Expected DatabaseError"),
        }
    }
    
    #[test]
    fn test_authentication_error() {
        let err = MilvusError::authentication("Invalid credentials");
        assert_eq!(err.category(), "authentication");
        assert!(err.is_client_error());
        assert!(!err.is_recoverable());
        assert_eq!(err.status_code(), 401);
    }
}
