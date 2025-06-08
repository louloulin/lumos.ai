//! Error types for the Lumosai framework

pub mod friendly;

use thiserror::Error;

/// Result type for Lumosai operations
pub type Result<T> = std::result::Result<T, Error>;

/// Alias for backward compatibility
pub type LumosError = Error;

/// Error types that can occur in Lumosai
#[derive(Error, Debug)]
pub enum Error {
    /// LLM provider errors
    #[error("LLM error: {0}")]
    Llm(String),

    /// LLM provider specific errors
    #[error("LLM provider error: {0}")]
    LlmProvider(String),

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
    
    /// Access denied errors
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    /// Invalid operation errors
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Security errors
    #[error("Security error: {0}")]
    SecurityError(String),

    /// Event system errors
    #[error("Event error: {0}")]
    Event(String),

    /// Vector store errors
    #[error("Vector store error: {0}")]
    VectorStore(String),

    /// RAG system errors
    #[error("RAG error: {0}")]
    Rag(String),

    /// Configuration errors (alias for Configuration)
    #[error("Config error: {0}")]
    Config(String),

    /// Configuration errors (structured)
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    /// Network errors (structured)
    #[error("Network error: {message}")]
    NetworkError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// API errors (structured)
    #[error("API error: {message}")]
    ApiError {
        message: String,
        status_code: Option<u16>,
    },

    /// Parse errors (structured)
    #[error("Parse error: {message}")]
    ParseError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Invalid state errors
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Timeout errors
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::Configuration(err.to_string())
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::Configuration(err)
    }
}