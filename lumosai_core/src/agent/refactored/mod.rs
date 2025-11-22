//! BasicAgent 重构模块
//!
//! 这个模块包含了 BasicAgent 重构后的组件，将原来的 BasicAgent 拆分为多个专门的组件：
//!
//! - `core::AgentCore` - Agent 核心配置和 LLM 提供者
//! - `executor::AgentExecutor` - Agent 执行器，包含工具和内存
//! - `generator::AgentGenerator` - Agent 生成器，包含工具解析器和内存解析器
//! - `agent::RefactoredAgent` - 统一的 Agent API 包装器
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
//! use lumosai_core::agent::refactored::RefactoredAgent;
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
//! let agent = RefactoredAgent::new(config, llm)?;
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

pub use agent::RefactoredAgent;
pub use core::AgentCore;
pub use executor::AgentExecutor;
pub use generator::AgentGenerator;

/// 创建重构后的 Agent（便捷函数）
///
/// 这是 `RefactoredAgent::new` 的便捷包装，提供与 `create_basic_agent` 类似的 API。
///
/// # 参数
///
/// * `name` - Agent 名称
/// * `instructions` - Agent 指令
/// * `llm` - LLM 提供者
///
/// # 返回
///
/// 返回 `Result<RefactoredAgent>`。
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::refactored::create_refactored_agent;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// # async fn example() -> lumosai_core::Result<()> {
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
/// let agent = create_refactored_agent("assistant", "You are helpful", llm)?;
/// # Ok(())
/// # }
/// ```
pub fn create_refactored_agent(
    name: impl Into<String>,
    instructions: impl Into<String>,
    llm: std::sync::Arc<dyn crate::llm::LlmProvider>,
) -> crate::error::Result<RefactoredAgent> {
    use crate::agent::AgentConfig;
    
    let config = AgentConfig {
        name: name.into(),
        instructions: instructions.into(),
        ..Default::default()
    };
    
    RefactoredAgent::new(config, llm)
}

/// 创建带内存的重构后的 Agent（便捷函数）
///
/// # 参数
///
/// * `name` - Agent 名称
/// * `instructions` - Agent 指令
/// * `llm` - LLM 提供者
/// * `memory` - 内存实例
///
/// # 返回
///
/// 返回 `Result<RefactoredAgent>`。
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::refactored::create_refactored_agent_with_memory;
/// use lumosai_core::llm::MockLlmProvider;
/// use lumosai_core::memory::BasicMemory;
/// use std::sync::Arc;
///
/// # async fn example() -> lumosai_core::Result<()> {
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
/// let memory = Arc::new(BasicMemory::new(None, None));
/// let agent = create_refactored_agent_with_memory("assistant", "You are helpful", llm, memory)?;
/// # Ok(())
/// # }
/// ```
pub fn create_refactored_agent_with_memory(
    name: impl Into<String>,
    instructions: impl Into<String>,
    llm: std::sync::Arc<dyn crate::llm::LlmProvider>,
    memory: std::sync::Arc<dyn crate::memory::Memory>,
) -> crate::error::Result<RefactoredAgent> {
    use crate::agent::AgentConfig;
    
    let config = AgentConfig {
        name: name.into(),
        instructions: instructions.into(),
        ..Default::default()
    };
    
    RefactoredAgent::with_memory(config, llm, memory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockLlmProvider;
    use crate::memory::BasicMemory;
    use std::sync::Arc;

    #[test]
    fn test_create_refactored_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        let agent = create_refactored_agent("test-agent", "You are helpful", llm).unwrap();
        
        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.instructions(), "You are helpful");
        assert!(!agent.has_memory());
    }

    #[test]
    fn test_create_refactored_agent_with_memory() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        let memory = Arc::new(BasicMemory::new(None, None));
        let agent = create_refactored_agent_with_memory("test-agent", "You are helpful", llm, memory).unwrap();
        
        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.instructions(), "You are helpful");
        assert!(agent.has_memory());
    }
}

