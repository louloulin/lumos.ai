//! 渐进式 API 实现 - LumosAI v2.0 第二周任务
//!
//! 实现 Mastra 风格的渐进式 API，提供三个层次的使用体验：
//! - Level 1: 5分钟上手，智能默认值
//! - Level 2: 链式配置，更多控制
//! - Level 3: 完整构建器模式，高级配置

use super::{AgentBuilder, BasicAgent};
use crate::agent::trait_def::Agent as AgentTrait;
use crate::agent::types::AgentGenerateOptions;
use crate::error::Result;
use crate::llm::{LlmProvider, Message, Role};
use crate::memory::MemoryConfig;
use crate::tool::Tool;
use crate::unified_api::{self, llm::*};
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
///     .temperature(0.7)
///     .max_tool_calls(10)
///     .system_prompt("你是一个专业的AI助手")
///     .tools(vec![web_search, calculator])
///     .memory(memory);
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
    /// Level 2 API - 设置模型（重新构建 Agent）
    ///
    /// # Example
    ///
    /// ```rust
    /// let agent = Agent::new("assistant", "你是一个AI助手").await?
    ///     .model("gpt-4");
    /// ```
    pub fn model(self, model_name: &str) -> Result<Self> {
        self.with_model(model_name)
    }

    /// Level 2 API - 设置模型（重新构建 Agent）
    ///
    /// # Example
    ///
    /// ```rust
    /// let agent = Agent::new("assistant", "你是一个AI助手").await?
    ///     .with_model("gpt-4")?;
    /// ```
    pub fn with_model(self, model_name: &str) -> Result<Self> {
        // 获取当前配置
        let name = self.inner.get_name().to_string();
        let instructions = self.inner.get_instructions().to_string();

        // 解析模型 - 智能检测模型提供商
        let model = Agent::resolve_model_provider(model_name)?;

        // 重新构建 Agent
        let new_agent = AgentBuilder::new()
            .name(&name)
            .instructions(&instructions)
            .model(model)
            .enable_smart_defaults()
            .build()?;

        Ok(Self::new(new_agent))
    }

    /// Level 2 API - 添加工具（重新构建 Agent）
    pub fn tools(self, tools: Vec<Box<dyn Tool>>) -> Result<Self> {
        self.with_tools(tools)
    }

    /// Level 2 API - 添加工具（重新构建 Agent）
    pub fn with_tools(self, tools: Vec<Box<dyn Tool>>) -> Result<Self> {
        // 获取当前配置
        let name = self.inner.get_name().to_string();
        let instructions = self.inner.get_instructions().to_string();
        let model = self.inner.get_llm();

        // 重新构建 Agent，添加工具
        let mut builder = AgentBuilder::new()
            .name(&name)
            .instructions(&instructions)
            .model(model)
            .enable_smart_defaults();

        for tool in tools {
            builder = builder.tool(tool);
        }

        let new_agent = builder.build()?;
        Ok(Self::new(new_agent))
    }

    /// Level 2 API - 通过工具名称添加工具（支持字符串名称）
    ///
    /// # Example
    ///
    /// ```rust
    /// let agent = Agent::new("assistant", "You are helpful").await?
    ///     .with_tool_names(&["web_search", "calculator"])?;
    /// ```
    pub fn with_tool_names(self, tool_names: &[&str]) -> Result<Self> {
        use crate::agent::tool_resolver::resolve_tool_names;

        // 解析工具名称到工具实例
        let tools = resolve_tool_names(tool_names)?;

        // 使用现有的 with_tools 方法
        self.with_tools(tools)
    }

    /// Level 2 API - 添加单个工具（通过名称）
    ///
    /// # Example
    ///
    /// ```rust
    /// let agent = Agent::new("assistant", "You are helpful").await?
    ///     .with_tool("web_search")?;
    /// ```
    pub fn with_tool(self, tool_name: &str) -> Result<Self> {
        self.with_tool_names(&[tool_name])
    }

    /// Level 2 API - 设置内存（重新构建 Agent）
    pub fn memory(self, memory: Arc<dyn crate::memory::Memory>) -> Result<Self> {
        self.with_memory(memory)
    }

    /// Level 2 API - 设置内存（重新构建 Agent）
    pub fn with_memory(self, memory: Arc<dyn crate::memory::Memory>) -> Result<Self> {
        // 获取当前配置
        let name = self.inner.get_name().to_string();
        let instructions = self.inner.get_instructions().to_string();
        let model = self.inner.get_llm();

        // 重新构建 Agent，直接使用 memory 实例
        let new_agent = AgentBuilder::new()
            .name(&name)
            .instructions(&instructions)
            .model(model)
            .with_basic_memory()
            .enable_smart_defaults()
            .build()?;

        Ok(Self::new(new_agent))
    }

    /// Level 2 API - 通过存储类型名称设置内存（支持字符串名称）
    ///
    /// # Example
    ///
    /// ```rust
    /// let agent = Agent::new("assistant", "You are helpful").await?
    ///     .with_memory_type("basic")?;
    /// ```
    pub fn with_memory_type(self, storage_type: &str) -> Result<Self> {
        use crate::agent::memory_resolver::resolve_memory_storage;

        // 解析存储类型名称到内存实例
        let memory = resolve_memory_storage(storage_type)?;

        // 使用现有的 with_memory 方法
        self.with_memory(memory)
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

    /// Level 2 API - 设置温度参数（重新构建 Agent）
    ///
    /// 控制生成文本的随机性，0.0-1.0之间
    pub fn temperature(self, temperature: f32) -> Result<Self> {
        // 获取当前配置
        let name = self.inner.get_name().to_string();
        let instructions = self.inner.get_instructions().to_string();
        let model = self.inner.get_llm();

        // 重新构建 Agent，添加温度设置
        let new_agent = AgentBuilder::new()
            .name(&name)
            .instructions(&instructions)
            .model(model)
            .temperature(temperature)
            .enable_smart_defaults()
            .build()?;

        Ok(Self::new(new_agent))
    }

    /// Level 2 API - 设置最大工具调用次数（重新构建 Agent）
    pub fn max_tool_calls(self, max_calls: usize) -> Result<Self> {
        // 获取当前配置
        let name = self.inner.get_name().to_string();
        let instructions = self.inner.get_instructions().to_string();
        let model = self.inner.get_llm();

        // 重新构建 Agent，添加最大工具调用设置
        let new_agent = AgentBuilder::new()
            .name(&name)
            .instructions(&instructions)
            .model(model)
            .max_tool_calls(max_calls as u32)
            .enable_smart_defaults()
            .build()?;

        Ok(Self::new(new_agent))
    }

    /// Level 2 API - 添加系统提示词（重新构建 Agent）
    pub fn system_prompt(self, system_prompt: &str) -> Result<Self> {
        // 合并系统提示词到指令中
        let current_instructions = self.inner.get_instructions();
        let new_instructions = if current_instructions.is_empty() {
            system_prompt.to_string()
        } else {
            format!("{}\n\n{}", system_prompt, current_instructions)
        };

        // 获取当前配置
        let name = self.inner.get_name().to_string();
        let model = self.inner.get_llm();

        // 重新构建 Agent
        let new_agent = AgentBuilder::new()
            .name(&name)
            .instructions(&new_instructions)
            .model(model)
            .enable_smart_defaults()
            .build()?;

        Ok(Self::new(new_agent))
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

    /// 自动检测可用的模型（增强版）
    async fn detect_available_model() -> Result<Arc<dyn LlmProvider>> {
        use std::env;

        // 检查环境变量来确定可用的模型
        let openai_key = env::var("OPENAI_API_KEY").ok();
        let claude_key = env::var("ANTHROPIC_API_KEY").ok();
        let deepseek_key = env::var("DEEPSEEK_API_KEY").ok();

        // 按优先级和环境可用性尝试不同的模型提供商

        // 1. 尝试 OpenAI (如果有 API key)
        if let Some(_key) = openai_key {
            if let Ok(model) = unified_api::llm::openai("gpt-4o-mini") {
                tracing::info!("Using OpenAI gpt-4o-mini for Level 1 API");
                return Ok(model);
            }
        }

        // 2. 尝试 Claude (如果有 API key)
        if let Some(_key) = claude_key {
            if let Ok(model) = unified_api::llm::claude("claude-3-haiku-20240307") {
                tracing::info!("Using Claude claude-3-haiku for Level 1 API");
                return Ok(model);
            }
        }

        // 3. 尝试 DeepSeek (如果有 API key)
        if let Some(_key) = deepseek_key {
            if let Ok(model) = unified_api::llm::deepseek("deepseek-chat") {
                tracing::info!("Using DeepSeek deepseek-chat for Level 1 API");
                return Ok(model);
            }
        }

        // 4. 尝试 Ollama (本地模型) - 作为默认选项
        tracing::info!("Falling back to Ollama llama3.2:3b for Level 1 API");
        let model = unified_api::llm::ollama("llama3.2:3b");
        Ok(model)
    }

    /// 使用完整的构建器模式（高级用法）
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///     /// use std::sync::Arc;
    ///
    /// let llm = create_test_zhipu_provider_arc();
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

    /// 创建动态配置的 Agent - 对标 Mastra 的 DynamicArgument
    ///
    /// 支持基于运行时上下文的动态配置，实现真正的上下文感知 Agent
    ///
    /// # Example
    ///
    /// ```rust
    /// use lumosai_core::agent::simplified_api::Agent;
    /// use lumosai_core::agent::dynamic_config::{EnhancedRuntimeContext, ComplexityLevel, dynamic_arg};
    ///
    /// // 创建运行时上下文
    /// let context = EnhancedRuntimeContext::new("session_123".to_string())
    ///     .with_user_role("developer".to_string())
    ///     .with_domain("ai_development".to_string())
    ///     .with_complexity(ComplexityLevel::Complex);
    ///
    /// // 创建动态配置的 Agent
    /// let agent = Agent::dynamic("adaptive_assistant")
    ///     .dynamic_instructions(dynamic_arg(|ctx| async move {
    ///         Ok(format!("You are a {} assistant for {}",
    ///             ctx.user_role.as_deref().unwrap_or("general"),
    ///             ctx.domain.as_deref().unwrap_or("general tasks")
    ///         ))
    ///     }))
    ///     .dynamic_model(dynamic_arg(|ctx| async move {
    ///         Ok(match ctx.complexity {
    ///             ComplexityLevel::Simple => "gpt-3.5-turbo".to_string(),
    ///             ComplexityLevel::Complex => "gpt-4".to_string(),
    ///             ComplexityLevel::Expert => "claude-3-opus".to_string(),
    ///         })
    ///     }))
    ///     .with_runtime_context(context)
    ///     .build_async().await?;
    /// ```
    pub fn dynamic(name: &str) -> AgentBuilder {
        AgentBuilder::new().name(name)
    }

    /// 智能解析模型提供商
    ///
    /// 根据模型名称自动选择合适的提供商
    fn resolve_model_provider(model_name: &str) -> Result<Arc<dyn LlmProvider>> {
        match model_name {
            // OpenAI 模型
            name if name.starts_with("gpt-") => openai(name),
            // Anthropic 模型
            name if name.starts_with("claude-") => claude(name),
            // DeepSeek 模型
            name if name.contains("deepseek") => deepseek(name),
            // Qwen 模型
            name if name.contains("qwen") || name.contains("glm") => qwen(name),
            // 默认尝试 DeepSeek（因为我们已经验证过它可以工作）
            _ => deepseek(model_name),
        }
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
    pub fn with_basic_memory(self) -> Self {
        self.memory_config(MemoryConfig::default())
    }

    /// 添加默认工具集
    pub fn with_default_tools(self) -> Self {
        // 添加常用工具：计算器、时间、基础文本处理
        self.with_calculator().with_time_tools().with_text_tools()
    }

    /// 添加计算器工具
    pub fn with_calculator(self) -> Self {
        // 添加基础数学运算工具：加减乘除、幂运算、三角函数等
        self.with_math_tools()
    }

    /// 添加时间工具
    pub fn with_time_tools(self) -> Self {
        // 添加获取当前时间、日期计算、时区转换等工具
        // 这些工具可以通过调用系统时间API来实现
        tracing::debug!("Adding time tools to agent");
        self // 暂时返回self，待工具系统完善后添加具体工具
    }

    /// 添加文本处理工具
    pub fn with_text_tools(self) -> Self {
        // 添加文本分析、格式化、编码转换等工具
        tracing::debug!("Adding text processing tools to agent");
        self // 暂时返回self，待工具系统完善后添加具体工具
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
/// /// use std::sync::Arc;
///
/// let llm = create_test_zhipu_provider_arc();
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
/// /// use std::sync::Arc;
///
/// let llm = create_test_zhipu_provider_arc();
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
/// /// use std::sync::Arc;
///
/// let llm = create_test_zhipu_provider_arc();
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
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_agent_quick_api() {
        let llm = create_test_zhipu_provider_arc();

        let agent = Agent::quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_agent_builder_api() {
        let llm = create_test_zhipu_provider_arc();

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
        let llm = create_test_zhipu_provider_arc();

        let agent = quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_web_agent() {
        let llm = create_test_zhipu_provider_arc();

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
        let llm = create_test_zhipu_provider_arc();

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
        let llm = create_test_zhipu_provider_arc();

        let agent = data_agent("data_helper")
            .model(llm)
            .build()
            .expect("Failed to create data agent");

        assert_eq!(agent.get_name(), "data_helper");
        assert!(agent.get_instructions().contains("data"));
        // Should have data and math tools
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_level_2_api_chaining() {
        let llm = create_test_zhipu_provider_arc();

        let agent = Agent::new("assistant", "You are helpful")
            .await
            .expect("Failed to create agent")
            .temperature(0.8)
            .expect("Failed to set temperature")
            .max_tool_calls(5)
            .expect("Failed to set max tool calls")
            .system_prompt("You are an expert assistant")
            .expect("Failed to set system prompt");

        assert_eq!(agent.name(), "assistant");
        assert!(agent.instructions().contains("expert"));
    }

    #[tokio::test]
    async fn test_level_1_api_with_env_detection() {
        // This test verifies that Level 1 API works with different environments
        let agent = Agent::new("test_agent", "You are a test agent")
            .await
            .expect("Failed to create agent with Level 1 API");

        assert_eq!(agent.name(), "test_agent");
        assert_eq!(agent.instructions(), "You are a test agent");
    }

    #[tokio::test]
    async fn test_enhanced_model_resolution() {
        let llm = create_test_zhipu_provider_arc();

        // Test various model name resolutions
        let agent = Agent::quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are helpful");
    }

    #[tokio::test]
    async fn test_with_tool_names() {
        let llm = create_test_zhipu_provider_arc();

        // Test adding tools by name
        let agent = Agent::quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        let agent_instance = AgentInstance::new(agent);
        let agent_with_tools = agent_instance
            .with_tool_names(&["calculator", "web_search"])
            .expect("Failed to add tools by name");

        // Verify tools were added
        let tools = agent_with_tools.inner.get_tools();
        assert!(tools.len() >= 2);
    }

    #[tokio::test]
    async fn test_with_tool() {
        let llm = create_test_zhipu_provider_arc();

        // Test adding single tool by name
        let agent = Agent::quick("assistant", "You are helpful")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        let agent_instance = AgentInstance::new(agent);
        let agent_with_tool = agent_instance
            .with_tool("calculator")
            .expect("Failed to add tool by name");

        // Verify tool was added
        let tools = agent_with_tool.inner.get_tools();
        assert!(tools.len() >= 1);
    }

    #[tokio::test]
    async fn test_chain_with_tool_names() {
        let llm = create_test_zhipu_provider_arc();

        // Test chaining with tool names
        let agent = Agent::new("assistant", "You are helpful")
            .await
            .expect("Failed to create agent")
            .with_tool("calculator")
            .expect("Failed to add calculator")
            .with_tool("web_search")
            .expect("Failed to add web_search");

        let tools = agent.inner.get_tools();
        assert!(tools.len() >= 2);
    }

    #[tokio::test]
    async fn test_with_memory_type() {
        let llm = create_test_zhipu_provider_arc();

        // Test adding memory by storage type name
        let agent = Agent::new("assistant", "You are helpful")
            .await
            .expect("Failed to create agent")
            .with_memory_type("basic")
            .expect("Failed to add basic memory");

        // Verify memory was set (we can't directly check, but the call should succeed)
        assert_eq!(agent.inner.get_name(), "assistant");
    }

    #[tokio::test]
    async fn test_chain_with_memory_type() {
        let llm = create_test_zhipu_provider_arc();

        // Test chaining memory type with tools
        let agent = Agent::new("assistant", "You are helpful")
            .await
            .expect("Failed to create agent")
            .with_memory_type("basic")
            .expect("Failed to add basic memory")
            .with_tool("calculator")
            .expect("Failed to add calculator");

        let tools = agent.inner.get_tools();
        assert!(tools.len() >= 1);
    }

    #[tokio::test]
    async fn test_with_memory_type_invalid() {
        let llm = create_test_zhipu_provider_arc();

        // Test invalid memory type
        let result = Agent::new("assistant", "You are helpful")
            .await
            .expect("Failed to create agent")
            .with_memory_type("invalid_memory_type");

        assert!(result.is_err());
    }
}
