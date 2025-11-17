//! # LumosAI A2A Protocol Implementation
//!
//! Google A2A (Agent-to-Agent) Protocol 的完整 Rust 实现。
//!
//! A2A 协议是 Google 提出的标准化的 Agent 间任务委托和协作协议，支持：
//!
//! - **Agent Cards**: 标准化的 Agent 能力声明
//! - **Task Delegation**: 跨 Agent 任务委托和执行
//! - **Artifact Exchange**: 任务结果的标准化交换
//! - **Service Discovery**: 动态 Agent 发现和匹配
//! - **Streaming Support**: 实时流式响应
//!
//! ## 快速开始
//!
//! ```rust
//! use lumosai_a2a::{AgentCard, AgentCardBuilder, A2AProtocol};
//!
//! // 创建 Agent Card
//! let card = AgentCardBuilder::new(
//!     "My Agent".to_string(),
//!     "https://my-agent.com".to_string(),
//!     "1.0.0".to_string(),
//! )
//! .description("A helpful AI agent".to_string())
//! .enable_streaming(true)
//! .skill(Skill::new(
//!     "text_analysis".to_string(),
//!     "Text Analysis".to_string(),
//! ))
//! .build()
//! .unwrap();
//!
//! // 创建 A2A 协议实例
//! let protocol = A2AProtocol::builder()
//!     .register_agent(card)
//!     .build();
//! ```
//!
//! ## 特性
//!
//! - **完整的 A2A 协议支持**: 100% 兼容 Google A2A 规范
//! - **类型安全**: 完整的 Rust 类型系统支持
//! - **异步优先**: 基于 Tokio 的异步 I/O
//! - **可扩展**: 支持自定义认证和传输层
//! - **生产就绪**: 完整的错误处理和日志记录
//!
//! ## 模块
//!
//! - [`client`]: A2A 客户端实现
//! - [`server`]: A2A 服务器实现  
//! - [`types`]: 核心数据类型
//! - [`card`]: Agent Card 管理
//! - [`task`]: 任务处理和状态管理
//! - [`discovery`]: Agent 发现和匹配

#![deny(missing_docs)]
#![warn(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod card;
pub mod client;
pub mod config;
pub mod discovery;
pub mod protocol;
pub mod server;
pub mod task;
pub mod types;

// 重新导出核心类型和功能
pub use card::{AgentCard, AgentCardBuilder, AgentCardManager, AgentCardStats};
pub use client::{A2AClient, A2AClientBuilder};
pub use config::{A2AConfig, A2ASettings};
pub use discovery::{AgentDiscovery, CapabilityMatcher};
pub use protocol::A2AProtocol;
pub use server::{A2AServer, A2AServerBuilder};
pub use task::{Task, TaskManager, TaskState, TaskStats};
pub use types::*;

/// A2A 协议版本
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// A2A 协议的默认端点路径
pub const DEFAULT_ENDPOINT: &str = "/a2a";

/// HTTP header constants
pub mod headers {
    /// A2A 协议版本头
    pub const A2A_VERSION: &str = "X-A2A-Version";
    
    /// Agent ID 头
    pub const AGENT_ID: &str = "X-Agent-ID";
    
    /// Task ID 头
    pub const TASK_ID: &str = "X-Task-ID";
    
    /// Content-Type for A2A messages
    pub const CONTENT_TYPE: &str = "application/json";
}

/// A2A 协议错误类型
#[derive(Debug, thiserror::Error)]
pub enum A2AError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    #[error("Capability not supported: {0}")]
    CapabilityNotSupported(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// A2A 协议结果类型
pub type A2AResult<T> = Result<T, A2AError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "1.0.0");
    }

    #[test]
    fn test_headers() {
        assert_eq!(headers::A2A_VERSION, "X-A2A-Version");
        assert_eq!(headers::AGENT_ID, "X-Agent-ID");
        assert_eq!(headers::TASK_ID, "X-Task-ID");
        assert_eq!(headers::CONTENT_TYPE, "application/json");
    }
}