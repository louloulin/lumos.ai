//! Agent 执行器组件 - BasicAgent 重构第二步
//!
//! 这个模块定义了 AgentExecutor，负责管理 Agent 的工具和内存。
//! 这是 BasicAgent 重构的第二步，将工具和内存管理从 BasicAgent 中分离出来。

use crate::agent::refactored::core::AgentCore;
use crate::agent::concurrent_tool_executor::ConcurrentToolExecutor;
use crate::agent::error_handling::RetryExecutor;
use crate::error::Result;
use crate::llm::LlmRouter;
use crate::memory::{create_working_memory, Memory, WorkingMemory};
use crate::tool::{Tool, ToolRegistry};
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
    /// 错误重试执行器（可选）
    retry_executor: Option<Arc<RetryExecutor>>,
    /// 并发工具执行器（可选）
    concurrent_tool_executor: Option<Arc<ConcurrentToolExecutor>>,
    /// LLM 路由器（可选）
    llm_router: Option<Arc<LlmRouter>>,
    /// 工具注册表（可选，用于工具发现和依赖解析）
    tool_registry: Option<Arc<ToolRegistry>>,
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
            retry_executor: None,
            concurrent_tool_executor: None,
            llm_router: None,
            tool_registry: None,
        })
    }

    /// 获取 Agent 核心（不可变引用）
    pub fn core(&self) -> &AgentCore {
        &self.core
    }

    /// 获取 Agent 核心（可变引用）
    pub fn core_mut(&mut self) -> &mut AgentCore {
        &mut self.core
    }

    /// 获取工具映射
    pub fn tools(&self) -> Arc<Mutex<HashMap<String, Box<dyn Tool>>>> {
        self.tools.clone()
    }

    /// 添加工具
    ///
    /// # 参数
    ///
    /// * `tool` - 要添加的工具
    ///
    /// # 返回
    ///
    /// 返回 `Result<()>`，如果添加失败则返回错误。
    pub fn add_tool(&self, tool: Box<dyn Tool>) -> Result<()> {
        let mut tools = self.tools.lock().map_err(|_| {
            crate::error::Error::Internal("Failed to lock tools mutex".to_string())
        })?;
        tools.insert(tool.id().to_string(), tool);
        Ok(())
    }

    /// 移除工具
    ///
    /// # 参数
    ///
    /// * `tool_name` - 要移除的工具名称
    ///
    /// # 返回
    ///
    /// 返回 `Result<()>`，如果工具不存在则返回错误。
    pub fn remove_tool(&self, tool_name: &str) -> Result<()> {
        let mut tools = self.tools.lock().map_err(|_| {
            crate::error::Error::Internal("Failed to lock tools mutex".to_string())
        })?;
        
        if !tools.contains_key(tool_name) {
            return Err(crate::error::Error::NotFound(
                format!("Tool '{}' not found", tool_name)
            ));
        }
        
        tools.remove(tool_name);
        Ok(())
    }

    /// 获取工具
    ///
    /// # 参数
    ///
    /// * `tool_name` - 工具名称
    ///
    /// # 返回
    ///
    /// 返回 `Option<Box<dyn Tool>>`，如果工具不存在则返回 `None`。
    pub fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>> {
        let tools = self.tools.lock().ok()?;
        tools.get(tool_name).cloned()
    }

    /// 列出所有工具名称
    ///
    /// # 返回
    ///
    /// 返回所有已注册的工具名称列表。
    pub fn list_tools(&self) -> Vec<String> {
        let tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        tools.keys().cloned().collect()
    }

    /// 检查工具是否存在
    ///
    /// # 参数
    ///
    /// * `tool_name` - 工具名称
    ///
    /// # 返回
    ///
    /// 如果工具存在返回 `true`，否则返回 `false`。
    pub fn has_tool(&self, tool_name: &str) -> bool {
        let tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        tools.contains_key(tool_name)
    }

    /// 获取工具数量
    ///
    /// # 返回
    ///
    /// 返回已注册的工具数量。
    pub fn tool_count(&self) -> usize {
        let tools = match self.tools.lock() {
            Ok(guard) => guard,
            Err(_) => return 0,
        };
        tools.len()
    }

    /// 检查是否有工具
    ///
    /// # 返回
    ///
    /// 如果有工具返回 `true`，否则返回 `false`。
    pub fn has_tools(&self) -> bool {
        self.tool_count() > 0
    }

    /// 清空所有工具
    ///
    /// # 返回
    ///
    /// 返回 `Result<()>`，如果清空失败则返回错误。
    pub fn clear_tools(&self) -> Result<()> {
        let mut tools = self.tools.lock().map_err(|_| {
            crate::error::Error::Internal("Failed to lock tools mutex".to_string())
        })?;
        tools.clear();
        Ok(())
    }

    /// 批量添加工具
    ///
    /// # 参数
    ///
    /// * `tools` - 要添加的工具列表
    ///
    /// # 返回
    ///
    /// 返回 `Result<()>`，如果添加失败则返回错误。
    pub fn add_tools(&self, tools: Vec<Box<dyn Tool>>) -> Result<()> {
        let mut tools_map = self.tools.lock().map_err(|_| {
            crate::error::Error::Internal("Failed to lock tools mutex".to_string())
        })?;
        
        for tool in tools {
            tools_map.insert(tool.id().to_string(), tool);
        }
        
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

    /// 设置错误重试执行器
    pub fn with_retry_executor(mut self, retry_executor: Arc<RetryExecutor>) -> Self {
        self.retry_executor = Some(retry_executor);
        self
    }

    /// 获取错误重试执行器
    pub fn retry_executor(&self) -> Option<Arc<RetryExecutor>> {
        self.retry_executor.clone()
    }

    /// 设置并发工具执行器
    pub fn with_concurrent_tool_executor(mut self, executor: Arc<ConcurrentToolExecutor>) -> Self {
        self.concurrent_tool_executor = Some(executor);
        self
    }

    /// 获取并发工具执行器
    pub fn concurrent_tool_executor(&self) -> Option<Arc<ConcurrentToolExecutor>> {
        self.concurrent_tool_executor.clone()
    }

    /// 设置 LLM 路由器
    ///
    /// # 参数
    ///
    /// * `router` - LLM 路由器
    ///
    /// # 返回
    ///
    /// 返回新的 `AgentExecutor` 实例，包含路由器。
    pub fn with_llm_router(mut self, router: Arc<LlmRouter>) -> Self {
        self.llm_router = Some(router);
        self
    }

    /// 获取 LLM 路由器
    pub fn llm_router(&self) -> Option<Arc<LlmRouter>> {
        self.llm_router.clone()
    }

    /// 设置工具注册表
    ///
    /// # 参数
    ///
    /// * `registry` - 工具注册表
    ///
    /// # 返回
    ///
    /// 返回新的 `AgentExecutor` 实例，包含工具注册表。
    pub fn with_tool_registry(mut self, registry: Arc<ToolRegistry>) -> Self {
        self.tool_registry = Some(registry);
        self
    }

    /// 获取工具注册表
    pub fn tool_registry(&self) -> Option<Arc<ToolRegistry>> {
        self.tool_registry.clone()
    }

    /// 发现工具（使用工具注册表，如果配置了的话）
    ///
    /// # 参数
    ///
    /// * `pattern` - 工具名称模式（支持通配符 * 和 ?）
    ///
    /// # 返回
    ///
    /// 返回匹配的工具列表。
    ///
    /// # 注意
    ///
    /// 此方法需要配置 ToolRegistry 才能使用。如果没有配置，将返回错误。
    pub fn discover_tools(&self, pattern: &str) -> Result<Vec<Arc<dyn Tool>>> {
        if let Some(registry) = &self.tool_registry {
            registry.discover(pattern)
        } else {
            Err(crate::error::Error::Configuration(
                "ToolRegistry not configured. Use with_tool_registry() first.".to_string(),
            ))
        }
    }

    /// 解析工具依赖（使用工具注册表，如果配置了的话）
    ///
    /// # 参数
    ///
    /// * `tool_name` - 工具名称
    ///
    /// # 返回
    ///
    /// 返回工具及其所有依赖的列表（按依赖顺序）。
    ///
    /// # 注意
    ///
    /// 此方法需要配置 ToolRegistry 才能使用。如果没有配置，将返回错误。
    pub fn resolve_tool_dependencies(&self, tool_name: &str) -> Result<Vec<Arc<dyn Tool>>> {
        if let Some(registry) = &self.tool_registry {
            registry.resolve_dependencies(tool_name)
        } else {
            Err(crate::error::Error::Configuration(
                "ToolRegistry not configured. Use with_tool_registry() first.".to_string(),
            ))
        }
    }
}

impl std::fmt::Debug for AgentExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentExecutor")
            .field("core", &self.core)
            .field("tool_count", &self.tool_count())
            .field("has_memory", &self.memory.is_some())
            .field("has_working_memory", &self.working_memory.is_some())
            .field("has_retry_executor", &self.retry_executor.is_some())
            .field("has_concurrent_tool_executor", &self.concurrent_tool_executor.is_some())
            .field("has_llm_router", &self.llm_router.is_some())
            .field("has_tool_registry", &self.tool_registry.is_some())
            .finish_non_exhaustive()
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

    #[test]
    fn test_agent_executor_with_retry_executor() {
        use crate::agent::error_handling::{RetryExecutor, RetryStrategy, BackoffStrategy, AgentErrorType};
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        
        // 创建 RetryExecutor
        let strategy = RetryStrategy {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential {
                initial_delay_ms: 100,
                multiplier: 2.0,
            },
            retryable_errors: vec![
                AgentErrorType::LlmError,
                AgentErrorType::NetworkError,
            ],
            max_delay_ms: Some(5000),
        };
        let retry_executor = Arc::new(RetryExecutor::with_default_recovery(strategy));
        
        let executor = executor.with_retry_executor(retry_executor.clone());
        assert!(executor.retry_executor().is_some());
        assert!(Arc::ptr_eq(&executor.retry_executor().unwrap(), &retry_executor));
    }

    #[test]
    fn test_agent_executor_with_concurrent_tool_executor() {
        use crate::agent::concurrent_tool_executor::{ConcurrentToolExecutor, ConcurrentToolExecutorConfig};
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        
        // 创建 ConcurrentToolExecutor
        let concurrent_config = ConcurrentToolExecutorConfig {
            max_concurrency: 5,
            preserve_order: false,
            timeout_seconds: Some(30),
        };
        let concurrent_executor = Arc::new(ConcurrentToolExecutor::new(concurrent_config));
        
        let executor = executor.with_concurrent_tool_executor(concurrent_executor.clone());
        assert!(executor.concurrent_tool_executor().is_some());
        assert!(Arc::ptr_eq(&executor.concurrent_tool_executor().unwrap(), &concurrent_executor));
    }

    #[test]
    fn test_agent_executor_with_llm_router() {
        use crate::llm::{LlmRouter, RoutingStrategy};
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        
        // 创建 LlmRouter
        let provider1: Arc<dyn crate::llm::LlmProvider> = Arc::new(MockLlmProvider::new(vec!["Response 1".to_string()]));
        let provider2: Arc<dyn crate::llm::LlmProvider> = Arc::new(MockLlmProvider::new(vec!["Response 2".to_string()]));
        let providers = vec![provider1, provider2];
        let router = Arc::new(LlmRouter::new(providers).with_strategy(RoutingStrategy::RoundRobin));
        
        let executor = executor.with_llm_router(router.clone());
        assert!(executor.llm_router().is_some());
        assert!(Arc::ptr_eq(&executor.llm_router().unwrap(), &router));
    }

    #[test]
    fn test_agent_executor_with_tool_registry() {
        use crate::tool::ToolRegistry;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();
        
        // 创建 ToolRegistry
        let registry = Arc::new(ToolRegistry::new());
        
        let executor = executor.with_tool_registry(registry.clone());
        assert!(executor.tool_registry().is_some());
        assert!(Arc::ptr_eq(&executor.tool_registry().unwrap(), &registry));
    }

    #[test]
    fn test_agent_executor_tool_management() {
        use crate::tool::create_tool;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();

        // 测试添加工具
        let echo_tool = create_tool(
            "echo",
            "Echo a message",
            vec![("message", "string", "Message to echo", true)],
            |params| {
                let message = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                Ok(serde_json::json!({"echo": message}))
            },
        ).unwrap();

        executor.add_tool(Box::new(echo_tool)).unwrap();
        
        // 测试列出工具
        let tools = executor.list_tools();
        assert_eq!(tools.len(), 1);
        assert!(tools.contains(&"echo".to_string()));
        
        // 测试检查工具是否存在
        assert!(executor.has_tool("echo"));
        assert!(!executor.has_tool("nonexistent"));
        
        // 测试获取工具
        let tool = executor.get_tool("echo");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().id(), "echo");
        
        // 测试移除工具
        executor.remove_tool("echo").unwrap();
        assert!(!executor.has_tool("echo"));
        assert_eq!(executor.list_tools().len(), 0);
        
        // 测试移除不存在的工具
        let result = executor.remove_tool("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_executor_tool_utilities() {
        use crate::tool::create_tool;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let core = AgentCore::new(config, llm).unwrap();
        let executor = AgentExecutor::new(core).unwrap();

        // 测试工具数量
        assert_eq!(executor.tool_count(), 0);
        assert!(!executor.has_tools());

        // 添加工具
        let echo_tool = create_tool(
            "echo",
            "Echo a message",
            vec![("message", "string", "Message to echo", true)],
            |params| {
                let message = params
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("No message");
                Ok(serde_json::json!({"echo": message}))
            },
        ).unwrap();

        executor.add_tool(Box::new(echo_tool)).unwrap();
        
        // 测试工具数量
        assert_eq!(executor.tool_count(), 1);
        assert!(executor.has_tools());

        // 测试批量添加工具
        let tool1 = create_tool(
            "tool1",
            "Tool 1",
            vec![],
            |_params| Ok(serde_json::json!({"result": "tool1"})),
        ).unwrap();
        let tool2 = create_tool(
            "tool2",
            "Tool 2",
            vec![],
            |_params| Ok(serde_json::json!({"result": "tool2"})),
        ).unwrap();

        executor.add_tools(vec![Box::new(tool1), Box::new(tool2)]).unwrap();
        assert_eq!(executor.tool_count(), 3);

        // 测试清空工具
        executor.clear_tools().unwrap();
        assert_eq!(executor.tool_count(), 0);
        assert!(!executor.has_tools());
    }
}

