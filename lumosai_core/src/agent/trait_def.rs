//! Agent trait definition

use crate::compat::{ListenOptions, VoiceOptions, VoiceProvider};
use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::agent::config::AgentConfig;
use crate::agent::types::{
    AgentGenerateOptions, AgentGenerateResult, AgentStep, AgentStreamOptions, RuntimeContext,
    ToolCall,
};
use crate::base::Base;
use crate::error::{Error, Result};
use crate::llm::{LlmProvider, Message};
use crate::memory::working::WorkingMemory;
use crate::memory::Memory;
use crate::tool::Tool;
// use crate::compat::{ListenOptions, VoiceOptions, VoiceProvider};
use crate::workflow::Workflow;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

/// Agent状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// 初始化中
    Initializing,
    /// 就绪状态
    Ready,
    /// 运行中
    Running,
    /// 暂停状态
    Paused,
    /// 错误状态
    Error(String),
    /// 停止状态
    Stopped,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Initializing
    }
}

/// Trait for agents that support structured output generation
#[async_trait]
pub trait AgentStructuredOutput: Send + Sync {
    /// Generate structured output based on a schema
    async fn generate_structured<T: DeserializeOwned + Send + 'static>(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<T>;
}

/// Trait for agents that support voice input (speech-to-text)
#[async_trait]
pub trait AgentVoiceListener: Send + Sync {
    /// Convert speech to text using the agent's voice provider
    async fn listen(
        &self,
        audio: impl AsyncRead + Send + Unpin + 'static,
        options: &ListenOptions,
    ) -> Result<String>;
}

/// Trait for agents that support voice output (text-to-speech)
#[async_trait]
pub trait AgentVoiceSender: Send + Sync {
    /// Convert text to speech using the agent's voice provider
    async fn speak(
        &self,
        text: &str,
        options: &VoiceOptions,
    ) -> Result<BoxStream<'_, Result<Vec<u8>>>>;
}

/// Core trait defining the functionality of an AI agent
///
/// The `Agent` trait provides the fundamental interface for all agents in LumosAI.
/// It defines methods for configuration, tool management, memory access, and
/// interaction with LLM providers.
///
/// # Overview
///
/// An agent is an autonomous entity that can:
/// - Generate responses using an LLM
/// - Use tools to perform actions
/// - Maintain conversation history in memory
/// - Execute workflows
/// - Stream responses in real-time
///
/// # Implementations
///
/// The primary implementation is [`BasicAgent`](crate::agent::BasicAgent), which
/// provides a full-featured agent with tool calling, memory management, and
/// streaming support.
///
/// # Examples
///
/// ```rust
/// use lumosai_core::agent::{Agent, AgentBuilder};
/// use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
///
/// # tokio_test::block_on(async {
/// let llm = create_test_zhipu_provider_arc();
/// let agent = AgentBuilder::new()
///     .name("assistant")
///     .instructions("You are a helpful AI assistant")
///     .model(llm)
///     .build()
///     .expect("Failed to build agent");
///
/// // Get agent information
/// let name = agent.name().unwrap_or("unknown");
/// println!("Agent name: {}", name);
/// # });
/// ```
///
/// # Thread Safety
///
/// All agents must be `Send + Sync`, allowing them to be safely shared across
/// threads and used in async contexts.
///
/// # See Also
///
/// - [`AgentBuilder`](crate::agent::AgentBuilder) - Builder for creating agents
/// - [`Tool`](crate::tool::Tool) - Tool trait for extending agent capabilities
/// - [`Memory`](crate::memory::Memory) - Memory trait for conversation history
#[async_trait]
pub trait Agent: Base + Send + Sync {
    /// Returns the agent's name
    ///
    /// The name is used for identification in logs, metrics, and debugging.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// let name = agent.get_name();
    /// println!("Agent name: {}", name);
    /// # }
    /// ```
    fn get_name(&self) -> &str;

    /// Returns the agent's instructions (system prompt)
    ///
    /// Instructions define the agent's behavior, personality, and capabilities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// let instructions = agent.get_instructions();
    /// println!("Agent instructions: {}", instructions);
    /// # }
    /// ```
    fn get_instructions(&self) -> &str;

    /// Updates the agent's instructions
    ///
    /// This allows you to dynamically change the agent's behavior at runtime.
    ///
    /// # Arguments
    ///
    /// * `instructions` - New system prompt for the agent
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &mut dyn Agent) {
    /// agent.set_instructions("You are now a creative writing assistant".to_string());
    /// # }
    /// ```
    fn set_instructions(&mut self, instructions: String);

    /// Returns the LLM provider used by the agent
    ///
    /// The LLM provider handles communication with the underlying language model.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// let llm = agent.get_llm();
    /// // Use the LLM provider...
    /// # }
    /// ```
    fn get_llm(&self) -> Arc<dyn LlmProvider>;

    /// Returns the agent's memory, if configured
    ///
    /// Memory stores conversation history and can be used for context retrieval.
    ///
    /// # Returns
    ///
    /// - `Some(memory)` if the agent has memory configured
    /// - `None` if the agent has no memory
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// if let Some(memory) = agent.get_memory() {
    ///     println!("Agent has memory configured");
    /// } else {
    ///     println!("Agent has no memory");
    /// }
    /// # }
    /// ```
    fn get_memory(&self) -> Option<Arc<dyn Memory>>;

    /// Checks if the agent has its own memory instance
    ///
    /// Returns `true` if the agent owns its memory, `false` if it shares
    /// memory with other agents or has no memory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// if agent.has_own_memory() {
    ///     println!("Agent has its own memory");
    /// }
    /// # }
    /// ```
    fn has_own_memory(&self) -> bool;

    /// Returns the agent's working memory, if configured
    ///
    /// Working memory is a short-term memory buffer with limited capacity,
    /// useful for maintaining recent conversation context.
    ///
    /// # Returns
    ///
    /// - `Some(working_memory)` if configured
    /// - `None` if not configured (default implementation)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// if let Some(working_memory) = agent.get_working_memory() {
    ///     println!("Agent has working memory");
    /// }
    /// # }
    /// ```
    fn get_working_memory(&self) -> Option<Arc<dyn WorkingMemory>> {
        None
    }

    /// Returns all tools available to the agent
    ///
    /// Tools extend the agent's capabilities by allowing it to perform
    /// actions like web searches, file operations, calculations, etc.
    ///
    /// # Returns
    ///
    /// A HashMap mapping tool names to tool implementations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// let tools = agent.get_tools();
    /// println!("Agent has {} tools", tools.len());
    /// for (name, tool) in tools {
    ///     println!("- {}: {}", name, tool.description());
    /// }
    /// # }
    /// ```
    fn get_tools(&self) -> HashMap<String, Box<dyn Tool>>;

    /// Returns tools with runtime context for dynamic resolution
    ///
    /// This method allows tools to be resolved dynamically based on the
    /// runtime context, enabling context-aware tool selection.
    ///
    /// # Arguments
    ///
    /// * `context` - Runtime context for tool resolution
    ///
    /// # Returns
    ///
    /// A HashMap of available tools for the given context.
    ///
    /// # Default Implementation
    ///
    /// The default implementation returns static tools via `get_tools()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::{Agent, types::RuntimeContext};
    ///
    /// # async fn example(agent: &dyn Agent) -> lumosai_core::Result<()> {
    /// let context = RuntimeContext::default();
    /// let tools = agent.get_tools_with_context(&context).await?;
    /// println!("Available tools: {}", tools.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_tools_with_context(
        &self,
        context: &RuntimeContext,
    ) -> Result<HashMap<String, Box<dyn Tool>>> {
        // Default implementation returns static tools
        Ok(self.get_tools())
    }

    /// Adds a tool to the agent
    ///
    /// Tools can be added dynamically at runtime to extend the agent's
    /// capabilities.
    ///
    /// # Arguments
    ///
    /// * `tool` - Box-wrapped tool implementation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A tool with the same name already exists
    /// - The tool is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    /// use lumosai_core::tool::Tool;
    ///
    /// # fn example(agent: &mut dyn Agent) -> lumosai_core::Result<()> {
    /// // Assuming you have a custom tool
    /// // let my_tool = Box::new(MyCustomTool::new());
    /// // agent.add_tool(my_tool)?;
    /// # Ok(())
    /// # }
    /// ```
    fn add_tool(&mut self, tool: Box<dyn Tool>) -> Result<()>;

    /// Removes a tool from the agent
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool to remove
    ///
    /// # Errors
    ///
    /// Returns an error if the tool does not exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &mut dyn Agent) -> lumosai_core::Result<()> {
    /// agent.remove_tool("calculator")?;
    /// # Ok(())
    /// # }
    /// ```
    fn remove_tool(&mut self, tool_name: &str) -> Result<()>;

    /// Returns a specific tool by name
    ///
    /// # Arguments
    ///
    /// * `tool_name` - Name of the tool to retrieve
    ///
    /// # Returns
    ///
    /// - `Some(tool)` if the tool exists
    /// - `None` if the tool does not exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # fn example(agent: &dyn Agent) {
    /// if let Some(tool) = agent.get_tool("calculator") {
    ///     println!("Found tool: {}", tool.description());
    /// }
    /// # }
    /// ```
    fn get_tool(&self, tool_name: &str) -> Option<Box<dyn Tool>>;

    /// Returns available workflows for the agent
    ///
    /// Workflows are reusable sequences of operations that can be executed
    /// by the agent.
    ///
    /// # Arguments
    ///
    /// * `context` - Runtime context for workflow resolution
    ///
    /// # Returns
    ///
    /// A HashMap mapping workflow names to workflow implementations.
    ///
    /// # Default Implementation
    ///
    /// Returns an empty HashMap (no workflows).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::{Agent, types::RuntimeContext};
    ///
    /// # async fn example(agent: &dyn Agent) -> lumosai_core::Result<()> {
    /// let context = RuntimeContext::default();
    /// let workflows = agent.get_workflows(&context).await?;
    /// println!("Available workflows: {}", workflows.len());
    /// # Ok(())
    /// # }
    /// ```
    async fn get_workflows(
        &self,
        context: &RuntimeContext,
    ) -> Result<HashMap<String, Arc<dyn Workflow>>> {
        // Default implementation returns empty workflows
        Ok(HashMap::new())
    }

    /// Executes a workflow by name
    ///
    /// # Arguments
    ///
    /// * `workflow_name` - Name of the workflow to execute
    /// * `input` - Input data for the workflow
    /// * `context` - Runtime context
    ///
    /// # Returns
    ///
    /// The workflow's output as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The workflow does not exist
    /// - The workflow execution fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::{Agent, types::RuntimeContext};
    /// use serde_json::json;
    ///
    /// # async fn example(agent: &dyn Agent) -> lumosai_core::Result<()> {
    /// let context = RuntimeContext::default();
    /// let input = json!({"task": "analyze data"});
    /// let result = agent.execute_workflow("data_analysis", input, &context).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn execute_workflow(
        &self,
        workflow_name: &str,
        input: Value,
        context: &RuntimeContext,
    ) -> Result<Value> {
        let workflows = self.get_workflows(context).await?;
        if let Some(workflow) = workflows.get(workflow_name) {
            workflow.execute(input, context).await
        } else {
            Err(Error::NotFound(format!(
                "Workflow '{workflow_name}' not found"
            )))
        }
    }

    /// Parses the LLM response to extract tool calls
    ///
    /// This method analyzes the LLM's response to identify any tool calls
    /// that need to be executed.
    ///
    /// # Arguments
    ///
    /// * `response` - The LLM's response text
    ///
    /// # Returns
    ///
    /// A vector of tool calls extracted from the response.
    ///
    /// # Errors
    ///
    /// Returns an error if the response cannot be parsed.
    fn parse_tool_calls(&self, response: &str) -> Result<Vec<ToolCall>>;

    /// Executes a tool call and returns the result
    ///
    /// # Arguments
    ///
    /// * `tool_call` - The tool call to execute
    ///
    /// # Returns
    ///
    /// The tool's output as a JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The tool does not exist
    /// - The tool execution fails
    /// - The tool times out
    async fn execute_tool_call(&self, tool_call: &ToolCall) -> Result<Value>;

    /// Formats messages for the LLM provider
    ///
    /// This method prepares messages for submission to the LLM, applying
    /// any necessary transformations or formatting.
    ///
    /// # Arguments
    ///
    /// * `messages` - Input messages
    /// * `options` - Generation options
    ///
    /// # Returns
    ///
    /// Formatted messages ready for the LLM.
    fn format_messages(&self, messages: &[Message], options: &AgentGenerateOptions)
        -> Vec<Message>;

    /// Generates a title for a conversation
    ///
    /// Creates a concise title summarizing the conversation based on the
    /// user's message.
    ///
    /// # Arguments
    ///
    /// * `user_message` - The user's message
    ///
    /// # Returns
    ///
    /// A generated title string.
    ///
    /// # Errors
    ///
    /// Returns an error if title generation fails.
    async fn generate_title(&self, user_message: &Message) -> Result<String>;

    /// Returns instructions with runtime context for dynamic resolution
    ///
    /// Allows instructions to be dynamically generated or modified based on
    /// the runtime context.
    ///
    /// # Arguments
    ///
    /// * `context` - Runtime context
    ///
    /// # Returns
    ///
    /// The agent's instructions (potentially modified based on context).
    ///
    /// # Default Implementation
    ///
    /// Returns static instructions via `get_instructions()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::{Agent, types::RuntimeContext};
    ///
    /// # async fn example(agent: &dyn Agent) -> lumosai_core::Result<()> {
    /// let context = RuntimeContext::default();
    /// let instructions = agent.get_instructions_with_context(&context).await?;
    /// # Ok(())
    /// # }
    /// ```
    async fn get_instructions_with_context(&self, context: &RuntimeContext) -> Result<String> {
        // Default implementation returns static instructions
        Ok(self.get_instructions().to_string())
    }

    /// Generates a response given a set of messages
    ///
    /// This is the core method for agent interaction. It processes the input
    /// messages and generates a response, potentially calling tools if needed.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation history
    /// * `options` - Generation options (temperature, max_tokens, etc.)
    ///
    /// # Returns
    ///
    /// An `AgentGenerateResult` containing the response and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The LLM request fails
    /// - Tool execution fails
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::{Agent, types::AgentGenerateOptions};
    /// use lumosai_core::llm::{Message, Role};
    ///
    /// # async fn example(agent: &dyn Agent) -> lumosai_core::Result<()> {
    /// let messages = vec![Message {
    ///     role: Role::User,
    ///     content: "Hello!".to_string(),
    ///     metadata: None,
    ///     name: None,
    /// }];
    ///
    /// let options = AgentGenerateOptions::default();
    /// let result = agent.generate(&messages, &options).await?;
    /// println!("Response: {}", result.response);
    /// # Ok(())
    /// # }
    /// ```
    async fn generate(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult>;

    /// Generates a response with runtime context for dynamic resolution
    ///
    /// Extended version of `generate()` that accepts runtime context for
    /// dynamic tool and instruction resolution.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation history
    /// * `options` - Generation options
    /// * `context` - Runtime context
    ///
    /// # Returns
    ///
    /// An `AgentGenerateResult` containing the response and metadata.
    ///
    /// # Default Implementation
    ///
    /// Calls `generate()` and ignores the context.
    async fn generate_with_context(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        context: &RuntimeContext,
    ) -> Result<AgentGenerateResult> {
        // Default implementation ignores context
        self.generate(messages, options).await
    }

    /// Generates a simple response from text input
    ///
    /// Convenience method for quick interactions without manually constructing
    /// messages and options.
    ///
    /// # Arguments
    ///
    /// * `input` - User's text input
    ///
    /// # Returns
    ///
    /// The agent's response as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if generation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::Agent;
    ///
    /// # async fn example(agent: &dyn Agent) -> lumosai_core::Result<()> {
    /// let response = agent.generate_simple("What is Rust?").await?;
    /// println!("Response: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    async fn generate_simple(&self, input: &str) -> Result<String> {
        use crate::llm::{Message, Role};

        let message = Message {
            role: Role::User,
            content: input.to_string(),
            metadata: None,
            name: None,
        };

        let options = AgentGenerateOptions::default();
        let result = self.generate(&[message], &options).await?;

        Ok(result.response)
    }

    /// Generates a response with multi-step reasoning
    ///
    /// Allows the agent to break down complex tasks into multiple steps,
    /// potentially calling tools multiple times.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation history
    /// * `options` - Generation options
    /// * `max_steps` - Maximum number of reasoning steps (None = unlimited)
    ///
    /// # Returns
    ///
    /// An `AgentGenerateResult` containing the final response and all steps.
    ///
    /// # Default Implementation
    ///
    /// Uses single-step generation via `generate()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::{Agent, types::AgentGenerateOptions};
    /// use lumosai_core::llm::{Message, Role};
    ///
    /// # async fn example(agent: &dyn Agent) -> lumosai_core::Result<()> {
    /// let messages = vec![Message {
    ///     role: Role::User,
    ///     content: "Solve this complex problem...".to_string(),
    ///     metadata: None,
    ///     name: None,
    /// }];
    ///
    /// let options = AgentGenerateOptions::default();
    /// let result = agent.generate_with_steps(&messages, &options, Some(5)).await?;
    /// println!("Response: {}", result.response);
    /// # Ok(())
    /// # }
    /// ```
    async fn generate_with_steps(
        &self,
        messages: &[Message],
        options: &AgentGenerateOptions,
        max_steps: Option<u32>,
    ) -> Result<AgentGenerateResult> {
        // Default implementation uses single step
        self.generate(messages, options).await
    }

    /// Generate a response with memory thread integration
    async fn generate_with_memory(
        &self,
        messages: &[Message],
        thread_id: Option<String>,
        options: &AgentGenerateOptions,
    ) -> Result<AgentGenerateResult>;

    /// Stream a response given a set of messages
    async fn stream<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions,
    ) -> Result<BoxStream<'a, Result<String>>>;

    /// Stream with callbacks for advanced control
    async fn stream_with_callbacks<'a>(
        &'a self,
        messages: &'a [Message],
        options: &'a AgentStreamOptions,
        on_step_finish: Option<Box<dyn FnMut(AgentStep) + Send + 'a>>,
        on_finish: Option<Box<dyn FnOnce(AgentGenerateResult) + Send + 'a>>,
    ) -> Result<BoxStream<'a, Result<String>>>;

    /// Get the agent's voice provider if configured
    fn get_voice(&self) -> Option<Arc<dyn VoiceProvider>>;

    /// Set a voice provider for the agent
    fn set_voice(&mut self, voice: Arc<dyn VoiceProvider>);

    /// Get a value from working memory
    async fn get_memory_value(&self, key: &str) -> Result<Option<Value>> {
        if let Some(memory) = self.get_working_memory() {
            memory.get_value(key).await
        } else {
            Err(Error::Unsupported(
                "Working memory not enabled for this agent".to_string(),
            ))
        }
    }

    /// Set a value in working memory
    async fn set_memory_value(&self, key: &str, value: Value) -> Result<()> {
        if let Some(memory) = self.get_working_memory() {
            memory.set_value(key, value).await
        } else {
            Err(Error::Unsupported(
                "Working memory not enabled for this agent".to_string(),
            ))
        }
    }

    /// Delete a value from working memory
    async fn delete_memory_value(&self, key: &str) -> Result<()> {
        if let Some(memory) = self.get_working_memory() {
            memory.delete_value(key).await
        } else {
            Err(Error::Unsupported(
                "Working memory not enabled for this agent".to_string(),
            ))
        }
    }

    /// Clear the working memory
    async fn clear_memory(&self) -> Result<()> {
        if let Some(memory) = self.get_working_memory() {
            memory.clear().await
        } else {
            Err(Error::Unsupported(
                "Working memory not enabled for this agent".to_string(),
            ))
        }
    }

    // === 统一Agent接口扩展 ===

    /// 获取Agent的配置信息
    fn get_config(&self) -> &AgentConfig {
        // 默认实现，子类应该重写
        panic!("get_config not implemented")
    }

    /// 获取Agent的状态信息
    fn get_status(&self) -> AgentStatus {
        AgentStatus::Ready
    }

    /// 设置Agent的状态
    fn set_status(&mut self, _status: AgentStatus) -> Result<()> {
        Ok(())
    }

    /// 获取Agent的元数据
    fn get_metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    /// 设置Agent的元数据
    fn set_metadata(&mut self, _metadata: HashMap<String, String>) -> Result<()> {
        Ok(())
    }

    /// 验证Agent配置
    fn validate_config(&self) -> Result<()> {
        Ok(())
    }

    /// 重新加载Agent配置
    async fn reload_config(&mut self, _config: AgentConfig) -> Result<()> {
        Ok(())
    }

    /// 获取Agent的健康状态
    async fn health_check(&self) -> Result<HashMap<String, Value>> {
        let mut health = HashMap::new();
        health.insert(
            "status".to_string(),
            serde_json::to_value(self.get_status())?,
        );
        health.insert(
            "name".to_string(),
            Value::String(self.get_name().to_string()),
        );
        health.insert("has_memory".to_string(), Value::Bool(self.has_own_memory()));
        health.insert(
            "tools_count".to_string(),
            Value::Number(self.get_tools().len().into()),
        );
        Ok(health)
    }

    /// 获取Agent的性能指标
    async fn get_metrics(&self) -> Result<HashMap<String, Value>> {
        // 默认返回空指标，子类可以重写
        Ok(HashMap::new())
    }

    /// 重置Agent状态
    async fn reset(&mut self) -> Result<()> {
        self.clear_memory().await?;
        self.set_status(AgentStatus::Ready)?;
        Ok(())
    }

    // ========== SOP (Standard Operating Procedure) 方法 ==========

    /// 声明 Agent 关注的消息类型（SOP Watch 阶段）
    ///
    /// 返回此 Agent 想要订阅和响应的消息类型列表。
    /// 这是 SOP 机制的第一步：Agent 声明它的"观察范围"。
    ///
    /// # 返回值
    ///
    /// 消息类型标识符列表。空列表表示不参与 SOP 协作。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// fn sop_watch(&self) -> Vec<String> {
    ///     vec![
    ///         "research_request".to_string(),
    ///         "analysis_needed".to_string(),
    ///     ]
    /// }
    /// ```
    fn sop_watch(&self) -> Vec<String> {
        Vec::new() // 默认：不订阅任何消息
    }

    /// 思考如何响应消息（SOP Think 阶段）
    ///
    /// 接收匹配 watch 列表的消息，决定采取什么行动。
    /// 这是 SOP 机制的第二步：Agent 分析消息并做出决策。
    ///
    /// # 参数
    ///
    /// * `messages` - 匹配 watch 列表的消息
    ///
    /// # 返回值
    ///
    /// 决定执行的行动
    ///
    /// # 示例
    ///
    /// ```ignore
    /// async fn sop_think(&mut self, messages: Vec<SopMessage>) -> Result<AgentAction> {
    ///     if messages.is_empty() {
    ///         return Ok(AgentAction::NoOp);
    ///     }
    ///
    ///     let msg = &messages[0];
    ///     // 分析消息内容
    ///     Ok(AgentAction::Reply {
    ///         content: "Processing...".to_string()
    ///     })
    /// }
    /// ```
    async fn sop_think(
        &self,
        _messages: Vec<super::sop_types::SopMessage>,
    ) -> Result<super::sop_types::AgentAction> {
        Ok(super::sop_types::AgentAction::NoOp) // 默认：不执行任何操作
    }

    /// 执行决定的行动（SOP Act 阶段）
    ///
    /// 执行 think 阶段决定的行动，产生新的消息。
    /// 这是 SOP 机制的第三步：Agent 执行行动并产生输出。
    ///
    /// # 参数
    ///
    /// * `action` - 要执行的行动
    ///
    /// # 返回值
    ///
    /// 执行结果消息
    ///
    /// # 示例
    ///
    /// ```ignore
    /// async fn sop_act(&mut self, action: AgentAction) -> Result<SopMessage> {
    ///     match action {
    ///         AgentAction::Reply { content } => {
    ///             Ok(SopMessage::new(
    ///                 "response",
    ///                 self.get_name(),
    ///                 None,
    ///                 json!({"content": content})
    ///             ))
    ///         }
    ///         _ => Ok(SopMessage::broadcast("noop", self.get_name(), json!({})))
    ///     }
    /// }
    /// ```
    async fn sop_act(
        &self,
        _action: super::sop_types::AgentAction,
    ) -> Result<super::sop_types::SopMessage> {
        // 默认：产生一个空消息
        Ok(super::sop_types::SopMessage::broadcast(
            "noop",
            self.get_name(),
            serde_json::json!({}),
        ))
    }

    /// 检查 Agent 是否完成了 SOP 任务
    ///
    /// 用于判断 Agent 是否已经完成了它在 SOP 流程中的工作。
    ///
    /// # 返回值
    ///
    /// `true` 表示已完成，`false` 表示还需要继续工作
    fn sop_is_done(&self) -> bool {
        false // 默认：永不完成
    }
}
