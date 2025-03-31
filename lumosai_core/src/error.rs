//! Error types for the Lumosai framework

use thiserror::Error;

/// Result type for Lumosai operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in Lumosai
#[derive(Error, Debug)]
pub enum Error {
    /// LLM provider errors
    #[error("LLM error: {0}")]
    Llm(String),

    /// Agent errors
    #[error("Agent error: {0}")]
    Agent(String),

    /// Tool errors
    #[error("Tool error: {0}")]
    Tool(String),

    /// Memory errors
    #[error("Memory error: {0}")]
    Memory(String),

    /// Storage errors
    #[error("Storage error: {0}")]
    Storage(String),

    /// Workflow errors
    #[error("Workflow error: {0}")]
    Workflow(String),

    /// HTTP client errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic errors
    #[error("{0}")]
    Other(String),

    /// Lock errors
    #[error("Lock error: {0}")]
    Lock(String),

    /// Already exists errors
    #[error("Already exists error: {0}")]
    AlreadyExists(String),

    /// Not found errors
    #[error("Not found error: {0}")]
    NotFound(String),

    /// Invalid input errors
    #[error("Invalid input error: {0}")]
    InvalidInput(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Internal errors
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Unavailable resource errors
    #[error("Unavailable resource: {0}")]
    Unavailable(String),
    
    /// Unsupported feature or operation
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    
    /// Parsing errors
    #[error("Parsing error: {0}")]
    Parsing(String),
    
    /// Constraint violations
    #[error("Constraint violation: {0}")]
    Constraint(String),
    
    /// Schema errors
    #[error("Schema error: {0}")]
    SchemaError(String),
    
    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),
} 