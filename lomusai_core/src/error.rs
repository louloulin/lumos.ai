use thiserror::Error;

/// Error type used throughout the library
#[derive(Error, Debug)]
pub enum Error {
    /// Error when interacting with LLM providers
    #[error("LLM error: {0}")]
    LlmError(String),

    /// Error when executing tools
    #[error("Tool execution error: {0}")]
    ToolError(String),

    /// Error when working with agents
    #[error("Agent error: {0}")]
    AgentError(String),

    /// Error when working with workflows
    #[error("Workflow error: {0}")]
    WorkflowError(String),
    
    /// Error when working with memory
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
} 