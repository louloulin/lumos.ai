use thiserror::Error;
use std::io;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeError;

#[derive(Error, Debug, Clone)]
pub enum MCPError {
    #[error("IO error: {0}")]
    IOError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Failed to connect to MCP server: {0}")]
    ConnectionError(String),

    #[error("JSON-RPC protocol error: {0}")]
    JsonRpcError(String),

    #[error("Tool execution error: {0}")]
    ToolExecutionError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("MCP server timeout after {0} ms")]
    TimeoutError(u64),

    #[error("MCP server returned an error: {0}")]
    ServerError(String),

    #[error("Resource not found: {0}")]
    ResourceNotFoundError(String),

    #[error("MCP protocol error: {0}")]
    ProtocolError(String),

    #[error("Duplicate configuration detected: {0}")]
    DuplicateConfiguration(String),

    #[error("Other error: {0}")]
    Other(String),
}

impl From<ReqwestError> for MCPError {
    fn from(err: ReqwestError) -> Self {
        MCPError::HttpError(err.to_string())
    }
}

impl From<SerdeError> for MCPError {
    fn from(err: SerdeError) -> Self {
        MCPError::DeserializationError(err.to_string())
    }
}

impl From<io::Error> for MCPError {
    fn from(err: io::Error) -> Self {
        MCPError::IOError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MCPError>; 