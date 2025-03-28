//! Error types for the Lomusai framework

use thiserror::Error;

/// Result type for Lomusai operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in Lomusai
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
} 