use thiserror::Error;

/// Error type for store operations
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Failed to connect to store: {0}")]
    ConnectionError(String),

    #[error("Index operation failed: {0}")]
    IndexError(String),
    
    #[error("Query operation failed: {0}")]
    QueryError(String),
    
    #[error("Vector operation failed: {0}")]
    VectorError(String),
    
    #[error("Invalid filter: {0}")]
    FilterError(String),
    
    #[error("Store configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal store error: {0}")]
    InternalError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Feature not supported: {0}")]
    NotSupported(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<serde_json::Error> for StoreError {
    fn from(err: serde_json::Error) -> Self {
        StoreError::SerializationError(err.to_string())
    }
}

#[cfg(feature = "qdrant")]
impl From<qdrant_client::error::Error> for StoreError {
    fn from(err: qdrant_client::error::Error) -> Self {
        StoreError::InternalError(err.to_string())
    }
}

#[cfg(feature = "postgres")]
impl From<sqlx::Error> for StoreError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(e) => StoreError::QueryError(e.to_string()),
            sqlx::Error::RowNotFound => StoreError::QueryError("Row not found".to_string()),
            sqlx::Error::ColumnNotFound(col) => StoreError::QueryError(format!("Column not found: {}", col)),
            sqlx::Error::PoolTimedOut => StoreError::ConnectionError("Connection pool timeout".to_string()),
            _ => StoreError::InternalError(err.to_string()),
        }
    }
} 