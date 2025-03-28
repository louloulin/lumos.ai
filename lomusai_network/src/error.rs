//! Agent网络错误定义

use std::fmt;
use std::result;

/// 错误类型
#[derive(Debug)]
pub enum Error {
    /// Agent未找到
    AgentNotFound(String),
    /// 网络错误
    Network(String),
    /// 路由错误
    Routing(String),
    /// 拓扑错误
    Topology(String),
    /// 服务发现错误
    Discovery(String),
    /// 序列化/反序列化错误
    Serialization(String),
    /// IO错误
    Io(std::io::Error),
    /// 其他错误
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AgentNotFound(id) => write!(f, "Agent未找到: {}", id),
            Error::Network(msg) => write!(f, "网络错误: {}", msg),
            Error::Routing(msg) => write!(f, "路由错误: {}", msg),
            Error::Topology(msg) => write!(f, "拓扑错误: {}", msg),
            Error::Discovery(msg) => write!(f, "服务发现错误: {}", msg),
            Error::Serialization(msg) => write!(f, "序列化/反序列化错误: {}", msg),
            Error::Io(err) => write!(f, "IO错误: {}", err),
            Error::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Serialization(err.to_string())
    }
}

/// 结果类型
pub type Result<T> = result::Result<T, Error>; 