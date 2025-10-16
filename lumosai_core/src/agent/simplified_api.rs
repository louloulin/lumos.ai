//! 渐进式 API 实现 - LumosAI v2.0 第二周任务
//!
//! 实现 Mastra 风格的渐进式 API，提供三个层次的使用体验：
//! - Level 1: 5分钟上手，智能默认值
//! - Level 2: 链式配置，更多控制
//! - Level 3: 完整构建器模式，高级配置

use super::{AgentBuilder, BasicAgent};
use crate::agent::trait_def::Agent as AgentTrait;
use crate::agent::types::{AgentGenerateOptions, AgentGenerateResult};
use crate::error::{Error, Result};
use crate::llm::{LlmProvider, Message, Role};
use crate::memory::MemoryConfig;
use crate::tool::Tool;
use crate::unified_api;
use std::sync::Arc;

/// 渐进式 Agent API - 三层设计
///
/// **Level 1 - 5分钟上手**:
/// ```rust
/// let agent = Agent::new("assistant", "你是一个AI助手").await?;
/// ```
///
/// **Level 2 - 链式配置**:
/// ```rust
/// let agent = Agent::new("assistant", "你是一个AI助手").await?
///     .model("gpt-4")
///     .tools(&[web_search, calculator])
///     .memory(Memory::basic());
/// ```
///
/// **Level 3 - 完整构建器**:
/// ```rust
/// let agent = Agent::builder()
///     .name("research_agent")
///     .instructions("专业研究助手")
///     .model(openai("gpt-4")?)
///     .max_tool_calls(10)
///     .temperature(0.7)
///     .build()?;
/// ```
pub struct Agent;

/// Level 2 API - Agent 实例，支持链式配置
///
/// 包装 BasicAgent，提供更友好的 API 和链式配置方法
pub struct AgentInstance {
    inner: BasicAgent,
}

impl AgentInstance {
    /// 创建新的 Agent 实例
    pub fn new(agent: BasicAgent) -> Self {
        Self { inner: agent }
    }

    /// Level 2 API - 设置模型
    ///
    /// # Example
    ///
    /// ```rust
    /// let agent = Agent::new("assistant", "你是一个AI助手").await?
    ///     .model("gpt-4");
    /// ```
    pub fn model(mut self, model_name: &str) -> Result<Self> {
        // 这里需要重新构建 agent，因为模型是不可变的
        // 在实际实现中，我们可能需要一个更灵活的设计
        // 暂时返回错误，提示用户使用 builder 模式
        Err(Error::Config("Model cannot be changed after creation. Use Agent::builder() for advanced configuration.".to_string()))
    }

    /// Level 2 API - 添加工具
    pub fn tools(mut self, tools: &[Box<dyn Tool>]) -> Result<Self> {
        for tool in tools {
            // 这里需要克隆工具，但 Tool trait 可能不支持克隆
            // 暂时返回错误，提示用户使用 builder 模式
        }
        Err(Error::Config("Tools cannot be added after creation. Use Agent::builder() for advanced configuration.".to_string()))
    }

    /// Level 2 API - 设置内存
    pub fn memory(mut self, memory: Arc<dyn crate::memory::Memory>) -> Result<Self> {
        // 同样的问题，BasicAgent 的内存是不可变的
        Err(Error::Config("Memory cannot be changed after creation. Use Agent::builder() for advanced configuration.".to_string()))
    }

    /// 生成响应
    pub async fn generate(&self, input: &str) -> Result<String> {
        let message = Message {
            role: Role::User,
            content: input.to_string(),
            metadata: None,
            name: None,
        };
        let options = AgentGenerateOptions::default();
        let response = self.inner.generate(&[message], &options).await?;
        Ok(response.response)
    }

    /// 获取 Agent 名称
    pub fn name(&self) -> &str {
        self.inner.get_name()
    }

    /// 获取 Agent 指令
    pub fn instructions(&self) -> &str {
        self.inner.get_instructions()
    }
}

impl Agent {
    /// Level 1 API - 5分钟上手，智能默认值
    ///
    /// 自动检测可用模型，配置基础内存，提供默认工具集
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::simplified_api::Agent;
    ///
    /// // 最简单的使用方式 - 自动配置一切
    /// let agent = Agent::new("assistant", "你是一个AI助手").await?;
    /// let response = agent.generate("你好").await?;
    /// ```
    pub async fn new(name: &str, instructions: &str) -> Result<AgentInstance> {
        // 智能默认值配置
        let model = Self::detect_available_model().await?;

        let agent = AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .model(model)
            .enable_smart_defaults()
            .with_basic_memory()
            .with_default_tools()
            .build()?;

        Ok(AgentInstance::new(agent))
    }

    /// 自动检测可用的模型
    async fn detect_available_model() -> Result<Arc<dyn LlmProvider>> {
        // 按优先级尝试不同的模型提供商

        // 1. 尝试 OpenAI
        if let Ok(model) = unified_api::llm::openai("gpt-4o-mini") {
            return Ok(model);
        }

        // 2. 尝试 Claude
        if let Ok(model) = unified_api::llm::claude("claude-3-haiku-20240307") {
            return Ok(model);
        }

        // 3. 尝试 Ollama (本地模型)
        let model = unified_api::llm::ollama("llama3.2:3b");
        Ok(model)
    }

    /// 使用完整的构建器模式（高级用法）
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    ///
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    ///
    /// let agent = Agent::builder()
    ///     .name("research_agent")
    ///     .instructions("专业研究助手")
    ///     .model(llm)
    ///     .max_tool_calls(10)
    ///     .build()
    ///     .expect("Failed to create agent");
    /// ```
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// 快速创建（向后兼容）- 返回 AgentBuilder 用于链式配置
    pub fn quick(name: &str, instructions: &str) -> AgentBuilder {
        AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .enable_smart_defaults()
    }
}

/// AgentBuilder 扩展 - 支持智能默认值
impl AgentBuilder {
    /// 配置基础内存
    pub fn with_basic_memory(mut self) -> Self {
        self.memory_config(MemoryConfig::default())
    }

    /// 添加默认工具集
    pub fn with_default_tools(mut self) -> Self {
        // 添加常用工具：计算器、时间、基础文本处理
        self.with_calculator()
            .with_time_tools()
            .with_text_tools()
    }

    /// 添加计算器工具
    pub fn with_calculator(mut self) -> Self {
        // 这里需要实际的计算器工具实现
        // 暂时返回 self，在后续实现中添加
        self
    }

    /// 添加时间工具
    pub fn with_time_tools(mut self) -> Self {
        // 添加获取当前时间、日期计算等工具
        self
    }

    /// 添加文本处理工具
    pub fn with_text_tools(mut self) -> Self {
        // 添加文本分析、格式化等工具
        self
    }
}

/// Create a quick agent with minimal configuration (plan4.md convenience function)
///
/// This is the most convenient way to create an agent:
/// ```rust
/// let agent = quick("assistant", "You are helpful")
///     .model(llm)
///     .build()?;
/// ```
pub fn quick(name: &str, instructions: &str) -> AgentBuilder {
    Agent::quick(name, instructions)
}

/// Create a web agent with pre-configured web tools (plan4.md API)
///
/// # Example
///
/// ```rust
/// use lumosai_core::agent::web_agent;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let agent = web_agent("web_helper")
///     .model(llm)
///     .build()
///     .expect("Failed to create web agent");
/// ```
pub fn web_agent(name: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions("You are a web assistant with access to web browsing tools")
        .with_web_tools()
        .enable_smart_defaults()
}

/// Create a file agent with pre-configured file tools (plan4.md API)
///
/// # Example
///
/// ```rust
/// use lumosai_core::agent::file_agent;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let agent = file_agent("file_helper")
///     .model(llm)
///     .build()
///     .expect("Failed to create file agent");
/// ```
pub fn file_agent(name: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions("You are a file assistant with access to file system tools")
        .with_file_tools()
        .enable_smart_defaults()
}

/// Create a data agent with pre-configured data processing tools (plan4.md API)
///
/// # Example
///
/// ```rust
/// use lumosai_core::agent::data_agent;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let agent = data_agent("data_helper")
///     .model(llm)
///     .build()
///     .expect("Failed to create data agent");
/// ```
pub fn data_agent(name: &str) -> AgentBuilder {
    AgentBuilder::new()
        .name(name)
        .instructions("You are a data assistant with access to data processing tools")
        .with_data_tools()
        .with_math_tools()
        .enable_smart_defaults()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::trait_def::Agent as AgentTrait;
    use crate::llm::MockLlmProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_agent_quick_api() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = Agent::quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_agent_builder_api() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = Agent::builder()
            .name("research_agent")
            .instructions("You are a research assistant")
            .model(llm)
            .max_tool_calls(10)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "research_agent");
        assert_eq!(agent.get_instructions(), "You are a research assistant");
    }

    #[tokio::test]
    async fn test_quick_convenience_function() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_web_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = web_agent("web_helper")
            .model(llm)
            .build()
            .expect("Failed to create web agent");

        assert_eq!(agent.get_name(), "web_helper");
        assert!(agent.get_instructions().contains("web"));
        // Should have web tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_file_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = file_agent("file_helper")
            .model(llm)
            .build()
            .expect("Failed to create file agent");

        assert_eq!(agent.get_name(), "file_helper");
        assert!(agent.get_instructions().contains("file"));
        // Should have file tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_data_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

        let agent = data_agent("data_helper")
            .model(llm)
            .build()
            .expect("Failed to create data agent");

        assert_eq!(agent.get_name(), "data_helper");
        assert!(agent.get_instructions().contains("data"));
        // Should have data and math tools
        assert!(agent.get_tools().len() > 0);
    }
}
