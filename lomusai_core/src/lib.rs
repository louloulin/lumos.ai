//! Lomusai Core - Rust实现的AI应用框架核心库
//! 
//! 提供了Agent、工作流、工具、LLM接口等核心功能

pub mod types;
pub mod agent;
pub mod llm;
pub mod tool;
pub mod workflow;
pub mod memory;
pub mod telemetry;
pub mod error;

/// Re-export common types and traits
pub use error::Error;
pub use llm::LlmProvider;
pub use agent::Agent;
pub use tool::Tool;
pub use workflow::Workflow;
pub use memory::{Memory, MemoryConfig};

/// Result type used throughout the library
pub type Result<T> = std::result::Result<T, Error>; 