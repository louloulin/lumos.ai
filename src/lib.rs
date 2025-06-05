//! Lumosai - Rust版本的Mastra AI应用框架
//!
//! Lumosai 是基于 Mastra 的 AI 应用框架的 Rust 实现版本，
//! 提供了强大的 LLM 集成、工具调用、内存管理等功能。

pub use lumosai_core as core;

// Re-export core types for convenience
pub use lumosai_core::{
    agent::{AgentTrait as Agent, BasicAgent, AgentConfig, AgentGenerateOptions, AgentFactory},
    llm::{LlmProvider, Message, Role},
    tool::{Tool, FunctionTool},
    error::{Error, Result},
};

/// Current version of Lumosai
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
