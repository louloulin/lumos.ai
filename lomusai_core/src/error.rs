//! 错误处理模块

use std::fmt;
use std::io;

/// 框架中可能出现的错误类型
#[derive(Debug)]
pub enum Error {
    /// LLM相关错误
    Llm(String),
    
    /// 配置错误
    Configuration(String),
    
    /// API调用错误
    Api(String),
    
    /// 序列化/反序列化错误
    Serialization(String),
    
    /// IO错误
    Io(io::Error),
    
    /// 工具执行错误
    ToolExecution(String),
    
    /// 内存操作错误
    Memory(String),
    
    /// 工作流执行错误
    Workflow(String),
    
    /// 其他错误
    Other(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Llm(msg) => write!(f, "LLM error: {}", msg),
            Error::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            Error::Api(msg) => write!(f, "API error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::ToolExecution(msg) => write!(f, "Tool execution error: {}", msg),
            Error::Memory(msg) => write!(f, "Memory error: {}", msg),
            Error::Workflow(msg) => write!(f, "Workflow error: {}", msg),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, Error>; 