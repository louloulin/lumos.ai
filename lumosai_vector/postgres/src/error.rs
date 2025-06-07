//! Error types for PostgreSQL vector storage

use thiserror::Error;
use lumosai_vector_core::VectorError;

/// PostgreSQL-specific error types
#[derive(Error, Debug)]
pub enum PostgresError {
    /// Database connection error
    #[error("Database connection error: {0}")]
    Connection(String),
    
    /// SQL execution error
    #[error("SQL execution error: {0}")]
    Sql(String),
    
    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Pool error
    #[error("Connection pool error: {0}")]
    Pool(String),
    
    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    /// Index error
    #[error("Index error: {0}")]
    Index(String),
    
    /// pgvector extension error
    #[error("pgvector extension error: {0}")]
    PgVector(String),
}

/// Result type for PostgreSQL operations
pub type PostgresResult<T> = Result<T, PostgresError>;

impl From<PostgresError> for VectorError {
    fn from(err: PostgresError) -> Self {
        match err {
            PostgresError::Connection(msg) => VectorError::connection_failed(msg),
            PostgresError::Sql(msg) => VectorError::OperationFailed(msg),
            PostgresError::Migration(msg) => VectorError::OperationFailed(format!("Migration: {}", msg)),
            PostgresError::Serialization(msg) => VectorError::serialization(msg),
            PostgresError::Config(msg) => VectorError::OperationFailed(format!("Configuration: {}", msg)),
            PostgresError::Pool(msg) => VectorError::connection_failed(format!("Pool: {}", msg)),
            PostgresError::Transaction(msg) => VectorError::OperationFailed(format!("Transaction: {}", msg)),
            PostgresError::Index(msg) => VectorError::OperationFailed(format!("Index: {}", msg)),
            PostgresError::PgVector(msg) => VectorError::OperationFailed(format!("pgvector: {}", msg)),
        }
    }
}

impl From<sqlx::Error> for PostgresError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Configuration(msg) => PostgresError::Config(msg.to_string()),
            sqlx::Error::Database(db_err) => PostgresError::Sql(db_err.to_string()),
            sqlx::Error::Io(io_err) => PostgresError::Connection(io_err.to_string()),
            sqlx::Error::Tls(tls_err) => PostgresError::Connection(tls_err.to_string()),
            sqlx::Error::Protocol(msg) => PostgresError::Connection(msg),
            sqlx::Error::RowNotFound => PostgresError::Sql("Row not found".to_string()),
            sqlx::Error::TypeNotFound { type_name } => {
                PostgresError::Sql(format!("Type not found: {}", type_name))
            },
            sqlx::Error::ColumnIndexOutOfBounds { index, len } => {
                PostgresError::Sql(format!("Column index {} out of bounds (len: {})", index, len))
            },
            sqlx::Error::ColumnNotFound(name) => {
                PostgresError::Sql(format!("Column not found: {}", name))
            },
            sqlx::Error::ColumnDecode { index, source } => {
                PostgresError::Serialization(format!("Column decode error at index {}: {}", index, source))
            },
            sqlx::Error::Decode(source) => {
                PostgresError::Serialization(format!("Decode error: {}", source))
            },
            sqlx::Error::PoolTimedOut => PostgresError::Pool("Pool timed out".to_string()),
            sqlx::Error::PoolClosed => PostgresError::Pool("Pool closed".to_string()),
            sqlx::Error::WorkerCrashed => PostgresError::Pool("Worker crashed".to_string()),
            _ => PostgresError::Sql(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for PostgresError {
    fn from(err: serde_json::Error) -> Self {
        PostgresError::Serialization(err.to_string())
    }
}

impl From<std::num::ParseFloatError> for PostgresError {
    fn from(err: std::num::ParseFloatError) -> Self {
        PostgresError::Serialization(format!("Float parse error: {}", err))
    }
}

impl From<std::num::ParseIntError> for PostgresError {
    fn from(err: std::num::ParseIntError) -> Self {
        PostgresError::Serialization(format!("Integer parse error: {}", err))
    }
}

/// Helper trait for converting results
pub trait PostgresResultExt<T> {
    /// Convert to VectorError result
    fn into_vector_result(self) -> lumosai_vector_core::Result<T>;
}

impl<T> PostgresResultExt<T> for PostgresResult<T> {
    fn into_vector_result(self) -> lumosai_vector_core::Result<T> {
        self.map_err(|e| e.into())
    }
}

/// Helper function to check if error is related to missing pgvector extension
pub fn is_pgvector_missing_error(err: &PostgresError) -> bool {
    match err {
        PostgresError::Sql(msg) | PostgresError::PgVector(msg) => {
            msg.contains("vector") && (
                msg.contains("does not exist") ||
                msg.contains("unknown type") ||
                msg.contains("extension")
            )
        },
        _ => false,
    }
}

/// Helper function to check if error is related to table not existing
pub fn is_table_not_found_error(err: &PostgresError) -> bool {
    match err {
        PostgresError::Sql(msg) => {
            msg.contains("relation") && msg.contains("does not exist")
        },
        _ => false,
    }
}

/// Helper function to check if error is related to connection issues
pub fn is_connection_error(err: &PostgresError) -> bool {
    matches!(err, 
        PostgresError::Connection(_) | 
        PostgresError::Pool(_)
    )
}

/// Helper function to create a pgvector extension error
pub fn pgvector_extension_error() -> PostgresError {
    PostgresError::PgVector(
        "pgvector extension is not installed. Please install it with: CREATE EXTENSION vector;".to_string()
    )
}

/// Helper function to create a table not found error
pub fn table_not_found_error(table_name: &str) -> PostgresError {
    PostgresError::Sql(format!("Table '{}' does not exist", table_name))
}

/// Helper function to create an index creation error
pub fn index_creation_error(index_name: &str, reason: &str) -> PostgresError {
    PostgresError::Index(format!("Failed to create index '{}': {}", index_name, reason))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_conversion() {
        let postgres_err = PostgresError::Connection("test".to_string());
        let vector_err: VectorError = postgres_err.into();
        
        assert!(matches!(vector_err, VectorError::ConnectionFailed { .. }));
    }
    
    #[test]
    fn test_pgvector_error_detection() {
        let err = PostgresError::Sql("type \"vector\" does not exist".to_string());
        assert!(is_pgvector_missing_error(&err));
        
        let err = PostgresError::Sql("some other error".to_string());
        assert!(!is_pgvector_missing_error(&err));
    }
    
    #[test]
    fn test_table_not_found_detection() {
        let err = PostgresError::Sql("relation \"test_table\" does not exist".to_string());
        assert!(is_table_not_found_error(&err));
        
        let err = PostgresError::Sql("some other error".to_string());
        assert!(!is_table_not_found_error(&err));
    }
}
