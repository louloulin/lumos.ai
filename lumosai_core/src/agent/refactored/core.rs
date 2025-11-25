//! Agent 核心组件 - BasicAgent 重构第一步
//!
//! 这个模块定义了 AgentCore，负责管理 Agent 的核心配置和 LLM 提供者。
//! 这是 BasicAgent 重构的第一步，将核心配置从 BasicAgent 中分离出来。

use crate::agent::AgentConfig;
use crate::error::Result;
use crate::llm::LlmProvider;
use std::sync::Arc;

/// Agent 核心组件
///
/// 负责管理 Agent 的核心配置和 LLM 提供者。
/// 这是 Agent 的"身份"组件，定义了 Agent 是谁，使用什么 LLM。
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::refactored::core::AgentCore;
/// use lumosai_core::agent::AgentConfig;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// let config = AgentConfig {
///     name: "test-agent".to_string(),
///     instructions: "You are a helpful assistant.".to_string(),
///     ..Default::default()
/// };
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let core = AgentCore::new(config, llm)?;
/// ```
pub struct AgentCore {
    /// Agent 名称
    name: String,
    /// Agent 指令
    instructions: String,
    /// LLM 提供者
    llm: Arc<dyn LlmProvider>,
    /// Agent 配置
    config: AgentConfig,
}

impl AgentCore {
    /// 创建新的 Agent 核心
    ///
    /// # 参数
    ///
    /// * `config` - Agent 配置
    /// * `llm` - LLM 提供者
    ///
    /// # 返回
    ///
    /// 返回 `Result<AgentCore>`，如果配置无效则返回错误。
    pub fn new(config: AgentConfig, llm: Arc<dyn LlmProvider>) -> Result<Self> {
        // 验证配置
        if config.name.is_empty() {
            return Err(crate::error::Error::InvalidInput(
                "Agent name cannot be empty".to_string()
            ));
        }
        
        // 验证 LLM provider 不为空
        // Arc 本身不会为空，但我们可以检查是否有效
        
        Ok(Self {
            name: config.name.clone(),
            instructions: config.instructions.clone(),
            llm,
            config,
        })
    }

    /// 获取 Agent 名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取 Agent 指令
    pub fn instructions(&self) -> &str {
        &self.instructions
    }

    /// 设置 Agent 指令
    pub fn set_instructions(&mut self, instructions: String) {
        self.instructions = instructions;
    }

    /// 设置 Agent 名称
    ///
    /// # 参数
    ///
    /// * `name` - 新的 Agent 名称
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 获取 LLM 提供者
    pub fn llm(&self) -> &Arc<dyn LlmProvider> {
        &self.llm
    }

    /// 获取 Agent 配置
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
}

impl std::fmt::Debug for AgentCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentCore")
            .field("name", &self.name)
            .field("instructions", &self.instructions)
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;
    use crate::llm::MockLlmProvider;

    #[test]
    fn test_agent_core_creation() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config.clone(), llm.clone()).unwrap();
        assert_eq!(core.name(), "test-agent");
        assert_eq!(core.instructions(), "You are a helpful assistant.");
        assert_eq!(core.config().name, config.name);
    }

    #[test]
    fn test_agent_core_set_instructions() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "Initial instructions.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let mut core = AgentCore::new(config, llm).unwrap();
        core.set_instructions("Updated instructions.".to_string());
        assert_eq!(core.instructions(), "Updated instructions.");
    }

    #[test]
    fn test_agent_core_set_name() {
        let config = AgentConfig {
            name: "initial-name".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let mut core = AgentCore::new(config, llm).unwrap();
        core.set_name("updated-name".to_string());
        assert_eq!(core.name(), "updated-name");
    }

    #[test]
    fn test_agent_core_validation() {
        use crate::llm::MockLlmProvider;
        
        // 测试空名称验证
        let config = AgentConfig {
            name: "".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let result = AgentCore::new(config, llm);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name cannot be empty"));
    }
}

