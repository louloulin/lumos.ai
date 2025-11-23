//! BasicAgent 实现 - 模块化架构
//!
//! 这个模块提供了 BasicAgent 的实现，使用模块化架构：
//! - `AgentCore`: 管理核心配置和 LLM 提供者
//! - `AgentExecutor`: 管理工具和内存
//! - `AgentGenerator`: 协调生成逻辑
//!
//! 这是重构后的 BasicAgent，将原来 2300+ 行的单体实现拆分为多个专门的组件。

use crate::agent::refactored::{AgentCore, AgentExecutor, AgentGenerator};
use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult, AgentStreamOptions, AgentStep, RuntimeContext, ToolCall};
use crate::agent::{Agent, AgentConfig};
use crate::base::{Base, BaseComponent, ComponentConfig};
use crate::compat::{Component, VoiceProvider};
use crate::error::{Error, Result};
use crate::llm::{LlmProvider, Message, Role};
use crate::memory::{Memory, working::WorkingMemory};
use crate::tool::Tool;
use crate::workflow::Workflow;
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// BasicAgent 实现
///
/// 这是重构后的 BasicAgent，使用模块化架构：
/// - `AgentCore`: 管理核心配置和 LLM 提供者
/// - `AgentExecutor`: 管理工具和内存
/// - `AgentGenerator`: 协调生成逻辑
///
/// # 示例
///
/// ```rust
/// use lumosai_core::agent::BasicAgent;
/// use lumosai_core::agent::AgentConfig;
/// use lumosai_core::llm::{Message, Role, MockLlmProvider};
/// use std::sync::Arc;
///
/// # async fn example() -> lumosai_core::Result<()> {
/// let config = AgentConfig {
///     name: "test-agent".to_string(),
///     instructions: "You are a helpful assistant.".to_string(),
///     ..Default::default()
/// };
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let agent = BasicAgent::new(config, llm)?;
///
/// let messages = vec![Message {
///     role: Role::User,
///     content: "Hello!".to_string(),
///     metadata: None,
///     name: None,
/// }];
/// let result = agent.generate(&messages, &Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub struct BasicAgent {
    /// Agent 生成器
    generator: AgentGenerator,
    /// Base component for logging and telemetry
    base: BaseComponent,
}

impl BasicAgent {
    /// 创建新的 BasicAgent
    ///
    /// # 参数
    ///
    /// * `config` - Agent 配置
    /// * `llm` - LLM 提供者
    ///
    /// # 返回
    ///
    /// 返回 `Result<BasicAgent>`。
    pub fn new(config: AgentConfig, llm: Arc<dyn LlmProvider>) -> Result<Self> {
        let core = AgentCore::new(config.clone(), llm)?;
        let executor = AgentExecutor::new(core)?;
        let generator = AgentGenerator::new(executor);
        
        let component_config = ComponentConfig {
            name: Some(config.name.clone()),
            component: Component::Agent,
            log_level: None,
        };
        let base = BaseComponent::new(component_config);

        Ok(Self { generator, base })
    }

    /// 使用内存创建 Agent（静态方法）
    ///
    /// # 参数
    ///
    /// * `config` - Agent 配置
    /// * `llm` - LLM 提供者
    /// * `memory` - 内存实例
    ///
    /// # 返回
    ///
    /// 返回 `Result<BasicAgent>`。
    ///
    /// # 注意
    ///
    /// 这是静态方法，用于创建时直接指定内存。
    /// 如果需要在已有 Agent 上添加内存，请使用实例方法 `with_memory()`。
    pub fn new_with_memory(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
        memory: Arc<dyn Memory>,
    ) -> Result<Self> {
        let core = AgentCore::new(config.clone(), llm)?;
        let executor = AgentExecutor::new(core)?.with_memory(memory);
        let generator = AgentGenerator::new(executor);
        
        let component_config = ComponentConfig {
            name: Some(config.name.clone()),
            component: Component::Agent,
            log_level: None,
        };
        let base = BaseComponent::new(component_config);

        Ok(Self { generator, base })
    }

    /// 生成响应
    ///
    /// # 参数
    ///
    /// * `messages` - 输入消息列表
    /// * `options` - 生成选项
    ///
    /// # 返回
    ///
    /// 返回 `Result<AgentGenerateResult>`，包含生成的响应和元数据。
    pub async fn generate(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        self.generator.generate(messages, options).await
    }

    /// 流式生成响应
    ///
    /// # 参数
    ///
    /// * `messages` - 输入消息列表
    /// * `options` - 流式生成选项
    ///
    /// # 返回
    ///
    /// 返回一个流，每个元素是一个字符串块。
    pub async fn stream<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        self.generator.stream(messages, options).await
    }

    /// 添加工具
    ///
    /// # 参数
    ///
    /// * `tool` - 要添加的工具
    ///
    /// # 返回
    ///
    /// 返回 `Result<()>`。
    ///
    /// # 注意
    ///
    /// 这个方法不需要 `&mut self`，因为工具存储在 `Arc<Mutex<...>>` 中，可以安全地并发访问。
    pub fn add_tool(&self, tool: Box<dyn Tool>) -> Result<()> {
        self.generator.executor().add_tool(tool)
    }

    /// 获取 Agent 名称
    pub fn name(&self) -> &str {
        self.generator.executor().core().name()
    }

    /// 获取 Agent 指令
    pub fn instructions(&self) -> &str {
        self.generator.executor().core().instructions()
    }

    /// 获取 LLM 提供者
    pub fn llm(&self) -> Arc<dyn LlmProvider> {
        self.generator.executor().core().llm().clone()
    }

    /// 获取内存（如果已配置）
    pub fn memory(&self) -> Option<Arc<dyn Memory>> {
        self.generator.executor().memory()
    }

    /// 检查是否有内存
    pub fn has_memory(&self) -> bool {
        self.generator.executor().memory().is_some()
    }

    /// 获取工具列表
    ///
    /// # 返回
    ///
    /// 返回工具映射的 `Arc<Mutex<...>>`，可以用于查询或修改工具。
    pub fn tools(&self) -> Arc<Mutex<HashMap<String, Box<dyn Tool>>>> {
        self.generator.executor().tools()
    }

    /// Check if LLM supports structured output
    pub fn supports_structured_output(&self) -> bool {
        self.generator.executor().core().llm().supports_structured_output()
    }

    /// 使用内存创建 Agent（构建器方法）
    ///
    /// # 参数
    ///
    /// * `memory` - 内存实例
    ///
    /// # 返回
    ///
    /// 返回新的 `BasicAgent` 实例，包含内存。
    ///
    /// # 注意
    ///
    /// 这个方法会重新构建整个 Agent 结构，因此会消耗一些资源。
    /// 建议在创建 Agent 时就配置好所有需要的组件。
    pub fn with_memory(mut self, memory: Arc<dyn Memory>) -> Result<Self> {
        let core = self.generator.executor().core();
        let config = core.config().clone();
        let llm = core.llm().clone();
        
        let new_core = AgentCore::new(config, llm)?;
        let mut new_executor = AgentExecutor::new(new_core)?;
        new_executor = new_executor.with_memory(memory);
        
        // 复制现有配置
        if let Some(retry_executor) = self.generator.executor().retry_executor() {
            new_executor = new_executor.with_retry_executor(retry_executor);
        }
        if let Some(concurrent_executor) = self.generator.executor().concurrent_tool_executor() {
            new_executor = new_executor.with_concurrent_tool_executor(concurrent_executor);
        }
        if let Some(llm_router) = self.generator.executor().llm_router() {
            new_executor = new_executor.with_llm_router(llm_router);
        }
        if let Some(tool_registry) = self.generator.executor().tool_registry() {
            new_executor = new_executor.with_tool_registry(tool_registry);
        }
        
        let new_generator = AgentGenerator::new(new_executor);
        
        let component_config = ComponentConfig {
            name: Some(self.base.name().unwrap_or("agent").to_string()),
            component: Component::Agent,
            log_level: None,
        };
        let base = BaseComponent::new(component_config);
        
        Ok(Self {
            generator: new_generator,
            base,
        })
    }

    /// 使用工具注册表创建 Agent（构建器方法）
    ///
    /// # 参数
    ///
    /// * `registry` - 工具注册表
    ///
    /// # 返回
    ///
    /// 返回新的 `BasicAgent` 实例，包含工具注册表。
    ///
    /// # 注意
    ///
    /// 这个方法会重新构建整个 Agent 结构，因此会消耗一些资源。
    /// 建议在创建 Agent 时就配置好所有需要的组件。
    pub fn with_tool_registry(mut self, registry: Arc<crate::tool::ToolRegistry>) -> Result<Self> {
        let core = self.generator.executor().core();
        let config = core.config().clone();
        let llm = core.llm().clone();
        
        let new_core = AgentCore::new(config, llm)?;
        let mut new_executor = AgentExecutor::new(new_core)?;
        
        // 复制现有配置
        if let Some(memory) = self.generator.executor().memory() {
            new_executor = new_executor.with_memory(memory);
        }
        if let Some(retry_executor) = self.generator.executor().retry_executor() {
            new_executor = new_executor.with_retry_executor(retry_executor);
        }
        if let Some(concurrent_executor) = self.generator.executor().concurrent_tool_executor() {
            new_executor = new_executor.with_concurrent_tool_executor(concurrent_executor);
        }
        if let Some(llm_router) = self.generator.executor().llm_router() {
            new_executor = new_executor.with_llm_router(llm_router);
        }
        
        new_executor = new_executor.with_tool_registry(registry);
        let generator = AgentGenerator::new(new_executor);
        Ok(Self { generator, base: self.base.clone() })
    }

    /// 使用 LLM 路由器创建 Agent（构建器方法）
    ///
    /// # 参数
    ///
    /// * `router` - LLM 路由器
    ///
    /// # 返回
    ///
    /// 返回新的 `BasicAgent` 实例，包含 LLM 路由器。
    pub fn with_llm_router(mut self, router: Arc<crate::llm::LlmRouter>) -> Result<Self> {
        let core = self.generator.executor().core();
        let config = core.config().clone();
        let llm = core.llm().clone();
        
        let new_core = AgentCore::new(config, llm)?;
        let mut new_executor = AgentExecutor::new(new_core)?;
        
        // 复制现有配置
        if let Some(memory) = self.generator.executor().memory() {
            new_executor = new_executor.with_memory(memory);
        }
        if let Some(retry_executor) = self.generator.executor().retry_executor() {
            new_executor = new_executor.with_retry_executor(retry_executor);
        }
        if let Some(concurrent_executor) = self.generator.executor().concurrent_tool_executor() {
            new_executor = new_executor.with_concurrent_tool_executor(concurrent_executor);
        }
        if let Some(tool_registry) = self.generator.executor().tool_registry() {
            new_executor = new_executor.with_tool_registry(tool_registry);
        }
        
        new_executor = new_executor.with_llm_router(router);
        let generator = AgentGenerator::new(new_executor);
        Ok(Self { generator, base: self.base.clone() })
    }

    /// 使用重试执行器创建 Agent（构建器方法）
    ///
    /// # 参数
    ///
    /// * `retry_executor` - 重试执行器
    ///
    /// # 返回
    ///
    /// 返回新的 `BasicAgent` 实例，包含重试执行器。
    pub fn with_retry_executor(mut self, retry_executor: Arc<crate::agent::error_handling::RetryExecutor>) -> Result<Self> {
        let core = self.generator.executor().core();
        let config = core.config().clone();
        let llm = core.llm().clone();
        
        let new_core = AgentCore::new(config, llm)?;
        let mut new_executor = AgentExecutor::new(new_core)?;
        
        // 复制现有配置
        if let Some(memory) = self.generator.executor().memory() {
            new_executor = new_executor.with_memory(memory);
        }
        if let Some(concurrent_executor) = self.generator.executor().concurrent_tool_executor() {
            new_executor = new_executor.with_concurrent_tool_executor(concurrent_executor);
        }
        if let Some(llm_router) = self.generator.executor().llm_router() {
            new_executor = new_executor.with_llm_router(llm_router);
        }
        if let Some(tool_registry) = self.generator.executor().tool_registry() {
            new_executor = new_executor.with_tool_registry(tool_registry);
        }
        
        new_executor = new_executor.with_retry_executor(retry_executor);
        let generator = AgentGenerator::new(new_executor);
        Ok(Self { generator, base: self.base.clone() })
    }

    /// 使用并发工具执行器创建 Agent（构建器方法）
    ///
    /// # 参数
    ///
    /// * `concurrent_executor` - 并发工具执行器
    ///
    /// # 返回
    ///
    /// 返回新的 `BasicAgent` 实例，包含并发工具执行器。
    pub fn with_concurrent_tool_executor(mut self, concurrent_executor: Arc<crate::agent::concurrent_tool_executor::ConcurrentToolExecutor>) -> Result<Self> {
        let core = self.generator.executor().core();
        let config = core.config().clone();
        let llm = core.llm().clone();
        
        let new_core = AgentCore::new(config, llm)?;
        let mut new_executor = AgentExecutor::new(new_core)?;
        
        // 复制现有配置
        if let Some(memory) = self.generator.executor().memory() {
            new_executor = new_executor.with_memory(memory);
        }
        if let Some(retry_executor) = self.generator.executor().retry_executor() {
            new_executor = new_executor.with_retry_executor(retry_executor);
        }
        if let Some(llm_router) = self.generator.executor().llm_router() {
            new_executor = new_executor.with_llm_router(llm_router);
        }
        if let Some(tool_registry) = self.generator.executor().tool_registry() {
            new_executor = new_executor.with_tool_registry(tool_registry);
        }
        
        new_executor = new_executor.with_concurrent_tool_executor(concurrent_executor);
        let generator = AgentGenerator::new(new_executor);
        Ok(Self { generator, base: self.base.clone() })
    }
}

// 实现 Base trait
impl Base for BasicAgent {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> Component {
        self.base.component()
    }

    fn logger(&self) -> Arc<dyn crate::logger::Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn crate::logger::Logger>) {
        self.base.set_logger(logger);
    }

    fn telemetry(&self) -> Option<Arc<dyn crate::telemetry::TelemetrySink>> {
        self.base.telemetry()
    }

    fn set_telemetry(&mut self, telemetry: Arc<dyn crate::telemetry::TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

// 实现 Agent trait
#[async_trait]
impl Agent for BasicAgent {
    fn get_name(&self) -> &str {
        self.name()
    }

    fn get_instructions(&self) -> &str {
        self.instructions()
    }

    fn set_instructions(&mut self, instructions: String) {
        // 更新 core 中的 instructions
        // 注意：由于 core 是不可变的，我们需要重新构建整个结构
        // 为了简化，这里我们只记录日志，实际的 instructions 更新需要在重构时处理
        self.base.logger().debug(&format!(
            "Instructions update requested for agent '{}'",
            self.name()
        ));
        // TODO: 实现真正的 instructions 更新（需要重新构建 AgentCore）
    }

    fn get_llm(&self) -> Arc<dyn LlmProvider> {
        self.llm()
    }

    fn get_memory(&self) -> Option<Arc<dyn Memory>> {
        self.memory()
    }

    fn has_own_memory(&self) -> bool {
        self.has_memory()
    }

    fn get_working_memory(&self) -> Option<Arc<dyn WorkingMemory>> {
        // BasicAgent 目前不支持 working memory
        None
    }

    fn get_tools(&self) -> HashMap<String, Box<dyn Tool>> {
        match self.tools().lock() {
            Ok(guard) => {
                let mut tools_copy = HashMap::new();
                for (name, tool) in guard.iter() {
                    tools_copy.insert(name.clone(), tool.clone());
                }
                tools_copy
            }
            Err(_) => HashMap::new(),
        }
    }

    async fn get_tools_with_context(
        &self,
        _context: &RuntimeContext,
    ) -> Result<HashMap<String, Box<dyn Tool>>> {
        // 直接获取工具，避免调用 self.get_tools()
        // 在同步块中完成所有操作，避免生命周期问题
        let tools_arc = self.generator.executor().tools();
        let tools = {
            match tools_arc.lock() {
                Ok(guard) => {
                    let mut tools_copy = HashMap::new();
                    for (name, tool) in guard.iter() {
                        tools_copy.insert(name.clone(), tool.clone());
                    }
                    tools_copy
                }
                Err(_) => HashMap::new(),
            }
        };
        Ok(tools)
    }

    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()> {
        // 由于 add_tool 需要 &mut self，但我们的实现使用 Arc<Mutex<...>>，
        // 我们可以直接调用内部方法，使用内部可变性
        let tool_name = tool.name().unwrap_or("unknown").to_string();
        let tools_arc = self.tools();
        let mut tools = match tools_arc.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(Error::Internal("Failed to lock tools".to_string())),
        };

        if tools.contains_key(&tool_name) {
            return Err(Error::Tool(format!("Tool '{tool_name}' already exists")));
        }

        tools.insert(tool_name.clone(), tool);
        self.base.logger().debug(&format!(
            "Tool '{}' added to agent '{}'",
            tool_name, self.name()
        ));

        Ok(())
    }

    fn remove_tool(&mut self, tool_name: &str) -> Result<()> {
        let tools_arc = self.tools();
        let mut tools = match tools_arc.lock() {
            Ok(guard) => guard,
            Err(_) => return Err(Error::Internal("Failed to lock tools".to_string())),
        };

        if !tools.contains_key(tool_name) {
            return Err(Error::NotFound(format!("Tool '{tool_name}' not found")));
        }

        tools.remove(tool_name);
        self.base.logger().debug(&format!(
            "Tool '{}' removed from agent '{}'",
            tool_name, self.name()
        ));

        Ok(())
    }

    fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>> {
        match self.tools().lock() {
            Ok(tools) => tools.get(tool_name).cloned(),
            Err(_) => None,
        }
    }

    async fn get_workflows(
        &self,
        _context: &RuntimeContext,
    ) -> Result<HashMap<String, Arc<dyn Workflow>>> {
        // BasicAgent 目前不支持 workflows
        Ok(HashMap::new())
    }

    async fn execute_workflow(
        &self,
        workflow_name: &str,
        _input: Value,
        _context: &RuntimeContext,
    ) -> Result<Value> {
        Err(Error::NotFound(format!(
            "Workflow '{workflow_name}' not found"
        )))
    }

    fn parse_tool_calls(&self, _response: &str) -> Result<Vec<ToolCall>> {
        // 简化实现：BasicAgent 使用 LLM 的原生工具调用支持
        // 工具调用解析由 LLM provider 处理
        Ok(Vec::new())
    }

    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<Value> {
        let tool_name = &tool_call.name;
        let tool = self.get_tool(tool_name).ok_or_else(|| {
            Error::NotFound(format!("Tool '{tool_name}' not found"))
        })?;

        // Convert HashMap to Value
        let args = serde_json::to_value(&tool_call.arguments).unwrap_or_else(|_| serde_json::json!({}));
        let context = crate::tool::ToolExecutionContext::default();
        let options = crate::tool::ToolExecutionOptions::default();
        let result = tool.execute(args, context, &options).await?;

        Ok(result)
    }

    fn format_messages(&self, messages: &[Message], _options: &AgentGenerateOptions) -> Vec<Message> {
        // 简化实现：直接返回消息
        messages.to_vec()
    }

    async fn generate_title(&self, user_message: &Message) -> Result<String> {
        // 简化实现：使用用户消息的前 50 个字符作为标题
        let title = user_message.content.chars().take(50).collect::<String>();
        Ok(title)
    }

    async fn get_instructions_with_context(&self, _context: &RuntimeContext) -> Result<String> {
        // 直接获取指令，避免调用 self.get_instructions()
        Ok(self.generator.executor().core().instructions().to_string())
    }

    async fn generate(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        self.generator.generate(messages, options).await
    }

    async fn generate_with_context(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        _context: &RuntimeContext,
    ) -> Result<AgentGenerateResult> {
        // 默认实现忽略 context
        self.generator.generate(messages, options).await
    }

    async fn generate_simple(&self, input: &str) -> Result<String> {
        let message = Message {
            role: Role::User,
            content: input.to_string(),
            metadata: None,
            name: None,
        };

        let messages = vec![message];
        let options = AgentGenerateOptions::default();
        let result = self.generator.generate(&messages, &options).await?;

        Ok(result.response)
    }

    async fn generate_with_steps(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        _max_steps: Option<u32>,
    ) -> Result<AgentGenerateResult> {
        // BasicAgent 的 generate 已经支持多步骤生成
        self.generator.generate(messages, options).await
    }

    async fn generate_with_memory(
        &self,
        messages: &[Message],
        thread_id: Option<String>,
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult> {
        // 如果提供了 thread_id，将其添加到 options 中
        let mut options = options.clone();
        if let Some(tid) = thread_id {
            options.thread_id = Some(tid);
        }
        self.generator.generate(messages, &options).await
    }

    async fn stream<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        self.generator.stream(messages, options).await
    }

    async fn stream_with_callbacks<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions,
        _on_step_finish: Option<Box<dyn FnMut(AgentStep) + Send + 'a>>,
        _on_finish: Option<Box<dyn FnOnce(AgentGenerateResult) + Send + 'a>>,
    ) -> Result<BoxStream<'a, Result<String>>> {
        // 简化实现：忽略回调，直接返回流
        self.generator.stream(messages, options).await
    }

    fn get_voice(&self) -> Option<Arc<dyn VoiceProvider>> {
        // BasicAgent 目前不支持 voice
        None
    }

    fn set_voice(&mut self, _voice: Arc<dyn VoiceProvider>) {
        // BasicAgent 目前不支持 voice
        self.base.logger().warn("Voice provider setting is not supported by BasicAgent");
    }

    async fn get_memory_value(&self, _key: &str) -> Result<Option<Value>> {
        Err(Error::Unsupported(
            "Working memory not enabled for BasicAgent".to_string(),
        ))
    }

    async fn set_memory_value(&self, _key: &str, _value: Value) -> Result<()> {
        Err(Error::Unsupported(
            "Working memory not enabled for BasicAgent".to_string(),
        ))
    }

    async fn clear_memory(&self) -> Result<()> {
        Err(Error::Unsupported(
            "Working memory not enabled for BasicAgent".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentConfig;
    use crate::llm::MockLlmProvider;

    #[tokio::test]
    async fn test_basic_agent_creation() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.instructions(), "You are a helpful assistant.");
        assert!(!agent.has_memory());
    }

    #[tokio::test]
    async fn test_basic_agent_generate() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello! How can I help you?".to_string()]));

        let agent = BasicAgent::new(config, llm).unwrap();

        let messages = vec![Message {
            role: crate::llm::Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentGenerateOptions::default();
        let result = agent.generate(&messages, &options).await.unwrap();
        assert!(!result.response.is_empty());
    }

    #[tokio::test]
    async fn test_basic_agent_stream() {
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello! How can I help you? This is a longer response.".to_string()]));

        let agent = BasicAgent::new(config, llm).unwrap();

        let messages = vec![Message {
            role: crate::llm::Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentStreamOptions::default();
        let mut stream = agent.stream(&messages, &options).await.unwrap();

        let mut chunks = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.unwrap();
            chunks.push(chunk);
        }

        assert!(!chunks.is_empty());
        let full_response: String = chunks.join("");
        assert!(!full_response.is_empty());
    }

    #[tokio::test]
    async fn test_basic_agent_add_tool() {
        use crate::tool::create_tool;

        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = BasicAgent::new(config, llm).unwrap();

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

        // 测试 add_tool（现在不需要 &mut）
        agent.add_tool(Box::new(echo_tool)).unwrap();

        // 验证工具已添加
        let tools = agent.tools();
        let tools_guard = tools.lock().unwrap();
        assert!(tools_guard.contains_key("echo"));
    }

    #[tokio::test]
    async fn test_basic_agent_builder_methods() {
        use crate::agent::error_handling::{RetryExecutor, RetryStrategy, BackoffStrategy, AgentErrorType};
        use crate::agent::concurrent_tool_executor::{ConcurrentToolExecutor, ConcurrentToolExecutorConfig};
        use crate::llm::{LlmRouter, RoutingStrategy};
        use crate::tool::ToolRegistry;
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = BasicAgent::new(config.clone(), llm.clone()).unwrap();
        
        // 测试 with_retry_executor
        let strategy = RetryStrategy {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential {
                initial_delay_ms: 100,
                multiplier: 2.0,
            },
            retryable_errors: vec![AgentErrorType::LlmError],
            max_delay_ms: Some(5000),
        };
        let retry_executor = Arc::new(RetryExecutor::with_default_recovery(strategy));
        let agent = agent.with_retry_executor(retry_executor).unwrap();
        
        // 测试 with_concurrent_tool_executor
        let concurrent_config = ConcurrentToolExecutorConfig {
            max_concurrency: 5,
            preserve_order: false,
            timeout_seconds: Some(30),
        };
        let concurrent_executor = Arc::new(ConcurrentToolExecutor::new(concurrent_config));
        let agent = agent.with_concurrent_tool_executor(concurrent_executor).unwrap();
        
        // 测试 with_llm_router
        let providers = vec![llm.clone()];
        let router = Arc::new(LlmRouter::new(providers).with_strategy(RoutingStrategy::RoundRobin));
        let agent = agent.with_llm_router(router).unwrap();
        
        // 测试 with_tool_registry
        let registry = Arc::new(ToolRegistry::new());
        let agent = agent.with_tool_registry(registry).unwrap();
        
        // 验证所有配置都已应用
        assert!(agent.generator.executor().retry_executor().is_some());
        assert!(agent.generator.executor().concurrent_tool_executor().is_some());
        assert!(agent.generator.executor().llm_router().is_some());
        assert!(agent.generator.executor().tool_registry().is_some());
    }

    #[tokio::test]
    async fn test_basic_agent_implements_agent_trait() {
        use crate::agent::Agent;
        use crate::agent::types::{AgentGenerateOptions, RuntimeContext};
        
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = BasicAgent::new(config, llm).unwrap();
        
        // 测试 Agent trait 方法
        assert_eq!(agent.get_name(), "test-agent");
        assert_eq!(agent.get_instructions(), "You are a helpful assistant.");
        assert!(agent.get_llm().supports_function_calling());
        assert!(!agent.has_own_memory());
        assert_eq!(agent.get_tools().len(), 0);
        
        // 测试异步方法
        let messages = vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
            metadata: None,
            name: None,
        }];
        let options = AgentGenerateOptions::default();
        let result = agent.generate(&messages, &options).await.unwrap();
        assert!(!result.response.is_empty());
        
        // 测试 generate_simple
        let response = agent.generate_simple("Test").await.unwrap();
        assert!(!response.is_empty());
        
        // 测试 get_tools_with_context
        let context = RuntimeContext::default();
        let tools = agent.get_tools_with_context(&context).await.unwrap();
        assert_eq!(tools.len(), 0);
        
        // 测试 get_instructions_with_context
        let instructions = agent.get_instructions_with_context(&context).await.unwrap();
        assert_eq!(instructions, "You are a helpful assistant.");
    }
}

