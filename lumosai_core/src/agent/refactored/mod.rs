//! BasicAgent 模块化组件
//!
//! 这个模块包含了 BasicAgent 的模块化组件：
//!
//! - `core::AgentCore` - Agent 核心配置和 LLM 提供者
//! - `executor::AgentExecutor` - Agent 执行器，包含工具和内存
//! - `generator::AgentGenerator` - Agent 生成器，包含工具解析器和内存解析器
//! - `agent::BasicAgent` - 统一的 Agent API 包装器
//!
//! # 设计目标
//!
//! 1. **降低复杂度**: 将 2300+ 行的 BasicAgent 拆分为多个小模块
//! 2. **提高可测试性**: 每个组件可以独立测试
//! 3. **提高可维护性**: 每个组件职责单一，易于理解和修改
//! 4. **保持兼容性**: 重构后的 API 保持与 BasicAgent 兼容
//!
//! # 使用示例
//!
//! ```rust
//! use lumosai_core::agent::BasicAgent;
//! use lumosai_core::agent::AgentConfig;
//! use lumosai_core::llm::{Message, Role, MockLlmProvider};
//! use std::sync::Arc;
//!
//! # async fn example() -> lumosai_core::Result<()> {
//! let config = AgentConfig {
//!     name: "test-agent".to_string(),
//!     instructions: "You are a helpful assistant.".to_string(),
//!     ..Default::default()
//! };
//! let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
//!
//! let agent = BasicAgent::new(config, llm)?;
//!
//! let messages = vec![Message {
//!     role: Role::User,
//!     content: "Hello!".to_string(),
//!     metadata: None,
//!     name: None,
//! }];
//! let result = agent.generate(&messages, &Default::default()).await?;
//! # Ok(())
//! # }
//! ```

pub mod agent;
pub mod core;
pub mod executor;
pub mod generator;

pub use agent::BasicAgent;
pub use core::AgentCore;
pub use executor::AgentExecutor;
pub use generator::AgentGenerator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockLlmProvider;
    use crate::memory::BasicMemory;
    use std::sync::Arc;
}
