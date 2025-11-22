//! Agent 执行器组件 - BasicAgent 重构第二步
//!
//! 这个模块定义了 AgentExecutor，负责管理 Agent 的工具和内存。
//! 这是 BasicAgent 重构的第二步，将工具和内存管理从 BasicAgent 中分离出来。

use crate::agent::refactored::core::AgentCore;
use crate::error::Result;
use crate::memory::{create_working_memory, Memory, WorkingMemory};
use crate::tool::Tool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Agent 执行器
///
/// 负责管理 Agent 的工具和内存。
/// 这是 Agent 的"能力"组件，定义了 Agent 可以使用什么工具和内存。
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::refactored::{AgentCore, AgentExecutor};
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
/// let executor = AgentExecutor::new(core)?;
/// ```
pub struct AgentExecutor {
    /// Agent 核心
    core: AgentCore,
    /// 工具映射
    tools: Arc<Mutex<HashMap<String, Box<dyn Tool>>>>,
    /// 内存
    memory: Option<Arc<dyn Memory>>,
    /// 工作内存
    working_memory: Option<Box<dyn WorkingMemory>>,
}

impl AgentExecutor {
    /// 创建新的 Agent 执行器
    ///
    /// # 参数
    ///
    /// * `core` - Agent 核心
    ///
    /// # 返回
    ///
    /// 返回 `Result<AgentExecutor>`，如果初始化失败则返回错误。
    pub fn new(core: AgentCore) -> Result<Self> {
        // 初始化工作内存（如果配置了）
        let working_memory = if let Some(wm_config) = &core.config().working_memory {
            match create_working_memory(wm_config) {
                Ok(wm) => Some(wm),
                Err(_) => None, // 静默失败，继续执行
            }
        } else {
            None
        };

        // 初始化内存（如果配置了）
        let memory = if let Some(_memory_config) = &core.config().memory_config {
            // 创建基础内存
            let basic_memory = crate::memory::BasicMemory::new(
                working_memory.as_ref().map(|_wm| {
                    use crate::memory::BasicWorkingMemory;
                    Arc::new(BasicWorkingMemory::new(
                        crate::memory::WorkingMemoryConfig {
                            enabled: true,
                            template: None,
                            content_type: None,
                            max_capacity: Some(100),
                        },
                    )) as Arc<dyn crate::memory::WorkingMemory>
                }),
                None,
            );
            Some(Arc::new(basic_memory) as Arc<dyn Memory>)
        } else {
            None
        };

        Ok(Self {
            core,
            tools: Arc::new(Mutex::new(HashMap::new())),
            memory,
            working_memory,
        })
    }

    /// 获取 Agent 核心
    pub fn core(&self) -> &AgentCore {
        &self.core
    }

    /// 获取工具映射
    pub fn tools(&self) -> Arc<Mutex<HashMap<String, Box<dyn Tool>>>> {
        self.tools.clone()
    }

    /// 添加工具
    pub fn add_tool(&self, tool: Box<dyn Tool>) -> Result<()> {
        let mut tools = self.tools.lock().map_err(|_| {
            crate::error::Error::Internal("Failed to lock tools mutex".to_string())
        })?;
        tools.insert(tool.id().to_string(), tool);
        Ok(())
    }

    /// 获取内存
    pub fn memory(&self) -> Option<Arc<dyn Memory>> {
        self.memory.clone()
    }

    /// 设置内存
    pub fn with_memory(mut self, memory: Arc<dyn Memory>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// 获取工作内存
    pub fn working_memory(&self) -> Option<&Box<dyn WorkingMemory>> {
        self.working_memory.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;
    use crate::llm::MockLlmProvider;

    #[test]
    fn test_agent_executor_creation() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        assert!(executor.memory().is_none());
        assert!(executor.working_memory().is_none());
    }

    #[test]
    fn test_agent_executor_with_memory() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        
        let memory = crate::memory::BasicMemory::new(None, None);
        let executor = executor.with_memory(Arc::new(memory));
        assert!(executor.memory().is_some());
    }
}

