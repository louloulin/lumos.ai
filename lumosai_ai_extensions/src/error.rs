//! AI扩展错误处理

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiExtensionError {
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Image processing error: {0}")]
    ImageProcessing(String),
    
    #[error("Audio processing error: {0}")]
    AudioProcessing(String),
    
    #[error("Video processing error: {0}")]
    VideoProcessing(String),
    
    #[error("Document processing error: {0}")]
    DocumentProcessing(String),
    
    #[error("Reasoning error: {0}")]
    ReasoningError(String),
    
    #[error("Knowledge graph error: {0}")]
    KnowledgeGraphError(String),
    
    #[error("Inference error: {0}")]
    InferenceError(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, AiExtensionError>;
