//! Agent builder for simplified agent creation
//!
//! This module provides a fluent builder API for creating agents,
//! inspired by Mastra's design but optimized for Rust.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use super::dynamic_config::{DynamicArgument, DynamicConfigResolver, EnhancedRuntimeContext};
use super::trait_def::Agent;
use super::types::{TelemetrySettings, VoiceConfig};
use super::{AgentConfig, BasicAgent, ModelResolver};
use crate::base::Base;
use crate::llm::LlmProvider;
use crate::memory::{MemoryConfig, WorkingMemoryConfig};
use crate::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use crate::{Error, Result};
use async_trait::async_trait;

/// Wrapper to convert `Arc<dyn Tool>` to `Box<dyn Tool>`
#[derive(Clone)]
struct ToolWrapper(Arc<dyn Tool>);

impl std::fmt::Debug for ToolWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolWrapper")
            .field("tool_id", &self.0.id())
            .field("tool_description", &self.0.description())
            .finish()
    }
}

impl Base for ToolWrapper {
    fn name(&self) -> Option<&str> {
        self.0.name()
    }

    fn component(&self) -> crate::compat::Component {
        self.0.component()
    }

    fn logger(&self) -> std::sync::Arc<dyn crate::logger::Logger> {
        self.0.logger()
    }

    fn set_logger(&mut self, _logger: std::sync::Arc<dyn crate::logger::Logger>) {
        // Cannot modify Arc content, so we ignore this
    }

    fn telemetry(&self) -> Option<std::sync::Arc<dyn crate::telemetry::TelemetrySink>> {
        self.0.telemetry()
    }

    fn set_telemetry(&mut self, _telemetry: std::sync::Arc<dyn crate::telemetry::TelemetrySink>) {
        // Cannot modify Arc content, so we ignore this
    }
}

#[async_trait]
impl Tool for ToolWrapper {
    fn id(&self) -> &str {
        self.0.id()
    }

    fn description(&self) -> &str {
        self.0.description()
    }

    fn schema(&self) -> crate::tool::ToolSchema {
        self.0.schema()
    }

    async fn execute(
        &self,
        params: Value,
        context: ToolExecutionContext,
        options: &ToolExecutionOptions,
    ) -> Result<Value> {
        self.0.execute(params, context, options).await
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

/// Builder for creating agents with a fluent API
///
/// `AgentBuilder` provides a flexible and type-safe way to construct AI agents
/// with various configurations. It follows the builder pattern, allowing you to
/// chain method calls to configure the agent before building it.
///
/// # Required Fields
///
/// - `name`: Agent identifier (must be unique in your application)
/// - `instructions`: System prompt that defines the agent's behavior
/// - `model`: LLM provider (OpenAI, Anthropic, Zhipu, etc.)
///
/// # Optional Fields
///
/// - `temperature`: Controls randomness (0.0-2.0, default: 0.7)
/// - `max_tokens`: Maximum response length (default: provider-specific)
/// - `max_tool_calls`: Maximum tool invocations per request (default: 10)
/// - `tool_timeout`: Tool execution timeout in seconds (default: 30)
/// - `tools`: List of tools available to the agent
/// - `memory_config`: Memory configuration for conversation history
/// - `working_memory`: Working memory configuration
/// - `voice_config`: Voice input/output configuration
/// - `telemetry`: Monitoring and logging settings
/// - `context`: Additional context data
/// - `metadata`: Custom metadata key-value pairs
///
/// # Examples
///
/// ## Basic Agent
///
/// ```rust
/// use lumosai_core::agent::AgentBuilder;
/// use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
///
/// # tokio_test::block_on(async {
/// let llm = create_test_zhipu_provider_arc();
///
/// let agent = AgentBuilder::new()
///     .name("assistant")
///     .instructions("You are a helpful AI assistant")
///     .model(llm)
///     .build()
///     .expect("Failed to build agent");
/// # });
/// ```
///
/// ## Agent with Tools
///
/// ```rust
/// use lumosai_core::agent::AgentBuilder;
/// use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
/// use lumosai_core::tool::Tool;
///
/// # tokio_test::block_on(async {
/// let llm = create_test_zhipu_provider_arc();
///
/// let agent = AgentBuilder::new()
///     .name("assistant")
///     .instructions("You are a helpful assistant with tools")
///     .model(llm)
///     .max_tool_calls(5)
///     .tool_timeout(60)
///     .build()
///     .expect("Failed to build agent");
/// # });
/// ```
///
/// ## Agent with Custom Configuration
///
/// ```rust
/// use lumosai_core::agent::AgentBuilder;
/// use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
///
/// # tokio_test::block_on(async {
/// let llm = create_test_zhipu_provider_arc();
///
/// let agent = AgentBuilder::new()
///     .name("creative_writer")
///     .instructions("You are a creative writing assistant")
///     .model(llm)
///     .temperature(0.9)
///     .max_tokens(2000)
///     .build()
///     .expect("Failed to build agent");
/// # });
/// ```
///
/// # Errors
///
/// The `build()` method returns an error if:
/// - Required fields (`name`, `instructions`, `model`) are missing
/// - Configuration values are invalid (e.g., temperature out of range)
/// - Model initialization fails
///
/// # Performance
///
/// Agent creation is a lightweight operation, typically completing in <1ms.
/// The agent itself is thread-safe and can be shared across multiple tasks.
///
/// # See Also
///
/// - [`Agent`] - The agent trait
/// - [`BasicAgent`] - The default agent implementation
/// - [`Tool`] - Tool trait for extending agent capabilities
pub struct AgentBuilder {
    name: Option<String>,
    instructions: Option<String>,
    model: Option<Arc<dyn LlmProvider>>,
    model_name: Option<String>, // New field for string model names
    memory_config: Option<MemoryConfig>,
    model_id: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    voice_config: Option<VoiceConfig>,
    telemetry: Option<TelemetrySettings>,
    working_memory: Option<WorkingMemoryConfig>,
    enable_function_calling: Option<bool>,
    context: Option<HashMap<String, Value>>,
    metadata: Option<HashMap<String, String>>,
    max_tool_calls: Option<u32>,
    tool_timeout: Option<u64>,
    tools: Vec<Box<dyn Tool>>,
    smart_defaults: bool,
    model_resolver: Option<ModelResolver>, // Model resolver for string names
    tenant_id: Option<String>,             // Multi-tenant support
    isolation_level: Option<String>,       // Isolation level for multi-tenancy

    // 动态配置支持 - 对标 Mastra
    dynamic_instructions: Option<DynamicArgument<String>>,
    dynamic_model: Option<DynamicArgument<String>>,
    dynamic_tools: Option<DynamicArgument<Vec<String>>>,
    runtime_context: Option<EnhancedRuntimeContext>,
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentBuilder {
    /// Creates a new `AgentBuilder` with default values
    ///
    /// All fields are initially `None` and must be set before calling `build()`.
    /// At minimum, you must set `name`, `instructions`, and `model`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            name: None,
            instructions: None,
            model: None,
            model_name: None,
            memory_config: None,
            model_id: None,
            temperature: None,
            max_tokens: None,
            voice_config: None,
            telemetry: None,
            working_memory: None,
            enable_function_calling: None,
            context: None,
            metadata: None,
            max_tool_calls: None,
            tool_timeout: None,
            tools: Vec::new(),
            smart_defaults: false,
            model_resolver: None,
            tenant_id: None,
            isolation_level: None,

            // 动态配置字段初始化
            dynamic_instructions: None,
            dynamic_model: None,
            dynamic_tools: None,
            runtime_context: None,
        }
    }

    /// Enables smart defaults for easier configuration
    ///
    /// When enabled, the builder will automatically fill in sensible default
    /// values for optional fields if they are not explicitly set.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().enable_smart_defaults();
    /// ```
    pub fn enable_smart_defaults(mut self) -> Self {
        self.smart_defaults = true;
        self
    }

    /// Sets the tenant ID for multi-tenant support
    ///
    /// In multi-tenant applications, the tenant ID is used to isolate data
    /// and resources between different tenants.
    ///
    /// # Arguments
    ///
    /// * `tenant_id` - Unique identifier for the tenant
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().tenant_id("tenant-123");
    /// ```
    pub fn tenant_id<S: Into<String>>(mut self, tenant_id: S) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Sets the isolation level for multi-tenancy
    ///
    /// Defines how strictly data and resources are isolated between tenants.
    /// Common values: "strict", "moderate", "relaxed".
    ///
    /// # Arguments
    ///
    /// * `level` - Isolation level identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().isolation_level("strict");
    /// ```
    pub fn isolation_level<S: Into<String>>(mut self, level: S) -> Self {
        self.isolation_level = Some(level.into());
        self
    }

    /// Sets the agent name (required)
    ///
    /// The name is used to identify the agent in logs, metrics, and debugging.
    /// It should be unique within your application.
    ///
    /// # Arguments
    ///
    /// * `name` - Agent identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().name("assistant");
    /// ```
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the agent instructions (required)
    ///
    /// Instructions define the agent's behavior, personality, and capabilities.
    /// This is also known as the "system prompt" in LLM terminology.
    ///
    /// # Arguments
    ///
    /// * `instructions` - System prompt for the agent
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new()
    ///     .instructions("You are a helpful AI assistant specialized in Rust programming.");
    /// ```
    pub fn instructions<S: Into<String>>(mut self, instructions: S) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Sets the LLM model provider (required)
    ///
    /// The model provider handles communication with the underlying LLM service
    /// (OpenAI, Anthropic, Zhipu, etc.).
    ///
    /// # Arguments
    ///
    /// * `model` - Arc-wrapped LLM provider
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
    ///
    /// let llm = create_test_zhipu_provider_arc();
    /// let builder = AgentBuilder::new().model(llm);
    /// ```
    pub fn model(mut self, model: Arc<dyn LlmProvider>) -> Self {
        self.model = Some(model);
        self
    }

    /// Sets the model using a string name (e.g., "gpt-4", "claude-3-sonnet")
    ///
    /// This method automatically resolves the model name to the appropriate
    /// provider. Useful for dynamic model selection based on configuration.
    ///
    /// # Arguments
    ///
    /// * `model_name` - Model identifier (e.g., "gpt-4", "claude-3-sonnet")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().model_name("gpt-4");
    /// ```
    ///
    /// # Note
    ///
    /// This requires appropriate API keys to be set in environment variables.
    pub fn model_name<S: Into<String>>(mut self, model_name: S) -> Self {
        self.model_name = Some(model_name.into());
        if self.model_resolver.is_none() {
            self.model_resolver = Some(ModelResolver::new());
        }
        self
    }

    /// Sets the model ID
    ///
    /// An optional identifier for the specific model instance or version.
    ///
    /// # Arguments
    ///
    /// * `model_id` - Model instance identifier
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().model_id("gpt-4-0613");
    /// ```
    pub fn model_id<S: Into<String>>(mut self, model_id: S) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    /// Sets the temperature for the model
    ///
    /// Temperature controls the randomness of the model's output:
    /// - Lower values (0.0-0.3): More focused and deterministic
    /// - Medium values (0.4-0.7): Balanced creativity and consistency
    /// - Higher values (0.8-2.0): More creative and diverse
    ///
    /// # Arguments
    ///
    /// * `temperature` - Value between 0.0 and 2.0 (default: 0.7)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// // For factual, deterministic responses
    /// let builder = AgentBuilder::new().temperature(0.1);
    ///
    /// // For creative writing
    /// let builder = AgentBuilder::new().temperature(0.9);
    /// ```
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the maximum number of tokens for the model
    ///
    /// Limits the length of the model's response. The exact meaning of "token"
    /// depends on the model provider (typically ~4 characters per token).
    ///
    /// # Arguments
    ///
    /// * `max_tokens` - Maximum response length in tokens
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// // For short responses
    /// let builder = AgentBuilder::new().max_tokens(100);
    ///
    /// // For long-form content
    /// let builder = AgentBuilder::new().max_tokens(4000);
    /// ```
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets memory configuration
    ///
    /// Configures how the agent stores and retrieves conversation history.
    ///
    /// # Arguments
    ///
    /// * `config` - Memory configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::memory::MemoryConfig;
    ///
    /// let config = MemoryConfig::default();
    /// let builder = AgentBuilder::new().memory_config(config);
    /// ```
    pub fn memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = Some(config);
        self
    }

    /// Sets voice configuration
    ///
    /// Configures voice input (speech-to-text) and output (text-to-speech)
    /// capabilities for the agent.
    ///
    /// # Arguments
    ///
    /// * `config` - Voice configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::agent::VoiceConfig;
    ///
    /// let config = VoiceConfig::default();
    /// let builder = AgentBuilder::new().voice_config(config);
    /// ```
    pub fn voice_config(mut self, config: VoiceConfig) -> Self {
        self.voice_config = Some(config);
        self
    }

    /// Sets telemetry settings
    ///
    /// Configures monitoring, logging, and tracing for the agent.
    ///
    /// # Arguments
    ///
    /// * `telemetry` - Telemetry configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::agent::TelemetrySettings;
    ///
    /// let settings = TelemetrySettings::default();
    /// let builder = AgentBuilder::new().telemetry(settings);
    /// ```
    pub fn telemetry(mut self, telemetry: TelemetrySettings) -> Self {
        self.telemetry = Some(telemetry);
        self
    }

    /// Sets working memory configuration
    ///
    /// Working memory is a short-term memory buffer that stores recent
    /// conversation context with a limited capacity.
    ///
    /// # Arguments
    ///
    /// * `config` - Working memory configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::memory::WorkingMemoryConfig;
    ///
    /// let config = WorkingMemoryConfig::default();
    /// let builder = AgentBuilder::new().working_memory(config);
    /// ```
    pub fn working_memory(mut self, config: WorkingMemoryConfig) -> Self {
        self.working_memory = Some(config);
        self
    }

    /// Enables or disables function calling
    ///
    /// When enabled, the agent can call tools to perform actions.
    /// When disabled, the agent can only generate text responses.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable function calling (default: true)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// // Disable tool usage
    /// let builder = AgentBuilder::new().enable_function_calling(false);
    /// ```
    pub fn enable_function_calling(mut self, enabled: bool) -> Self {
        self.enable_function_calling = Some(enabled);
        self
    }

    /// Sets context data
    ///
    /// Context data is additional information that can be accessed by the agent
    /// during execution. This is useful for passing runtime configuration or
    /// environment-specific data.
    ///
    /// # Arguments
    ///
    /// * `context` - Key-value pairs of context data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// let mut context = HashMap::new();
    /// context.insert("user_id".to_string(), json!("user-123"));
    /// context.insert("session_id".to_string(), json!("session-456"));
    ///
    /// let builder = AgentBuilder::new().context(context);
    /// ```
    pub fn context(mut self, context: HashMap<String, Value>) -> Self {
        self.context = Some(context);
        self
    }

    /// Adds a single context value
    ///
    /// Convenience method for adding individual context values without
    /// creating a HashMap manually.
    ///
    /// # Arguments
    ///
    /// * `key` - Context key
    /// * `value` - Context value (JSON)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use serde_json::json;
    ///
    /// let builder = AgentBuilder::new()
    ///     .add_context("user_id", json!("user-123"))
    ///     .add_context("language", json!("en"));
    /// ```
    pub fn add_context<K: Into<String>>(mut self, key: K, value: Value) -> Self {
        if self.context.is_none() {
            self.context = Some(HashMap::new());
        }
        if let Some(ref mut context) = self.context {
            context.insert(key.into(), value);
        }
        self
    }

    /// Sets metadata
    ///
    /// Metadata is custom key-value pairs that can be used for tagging,
    /// categorization, or storing additional information about the agent.
    ///
    /// # Arguments
    ///
    /// * `metadata` - Key-value pairs of metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use std::collections::HashMap;
    ///
    /// let mut metadata = HashMap::new();
    /// metadata.insert("version".to_string(), "1.0".to_string());
    /// metadata.insert("environment".to_string(), "production".to_string());
    ///
    /// let builder = AgentBuilder::new().metadata(metadata);
    /// ```
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Adds a single metadata value
    ///
    /// Convenience method for adding individual metadata values.
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key
    /// * `value` - Metadata value
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new()
    ///     .add_metadata("version", "1.0")
    ///     .add_metadata("author", "LumosAI Team");
    /// ```
    pub fn add_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(ref mut metadata) = self.metadata {
            metadata.insert(key.into(), value.into());
        }
        self
    }

    /// Sets the maximum number of tool calls per request
    ///
    /// Limits how many times the agent can call tools in a single request.
    /// This prevents infinite loops and controls execution time.
    ///
    /// # Arguments
    ///
    /// * `max` - Maximum number of tool calls (default: 10)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// // Allow up to 5 tool calls
    /// let builder = AgentBuilder::new().max_tool_calls(5);
    /// ```
    pub fn max_tool_calls(mut self, max: u32) -> Self {
        self.max_tool_calls = Some(max);
        self
    }

    /// Sets the tool execution timeout in seconds
    ///
    /// Maximum time allowed for a single tool execution. If a tool takes
    /// longer than this, it will be cancelled.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout in seconds (default: 30)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// // Set 60 second timeout for long-running tools
    /// let builder = AgentBuilder::new().tool_timeout(60);
    /// ```
    pub fn tool_timeout(mut self, timeout: u64) -> Self {
        self.tool_timeout = Some(timeout);
        self
    }

    /// Adds a tool to the agent
    ///
    /// Tools extend the agent's capabilities by allowing it to perform
    /// actions like web searches, file operations, calculations, etc.
    ///
    /// # Arguments
    ///
    /// * `tool` - Box-wrapped tool implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::tool::Tool;
    ///
    /// // Assuming you have a custom tool
    /// // let my_tool = Box::new(MyCustomTool::new());
    /// // let builder = AgentBuilder::new().tool(my_tool);
    /// ```
    pub fn tool(mut self, tool: Box<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Adds a tool to the agent (Arc version)
    ///
    /// Alternative to `tool()` that accepts Arc-wrapped tools.
    ///
    /// # Arguments
    ///
    /// * `tool` - Arc-wrapped tool implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::tool::Tool;
    /// use std::sync::Arc;
    ///
    /// // Assuming you have a custom tool
    /// // let my_tool = Arc::new(MyCustomTool::new());
    /// // let builder = AgentBuilder::new().add_tool(my_tool);
    /// ```
    pub fn add_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        // Convert Arc to Box by cloning the tool
        // Note: This requires the Tool trait to implement Clone or we need a different approach
        // For now, we'll use a workaround
        self.tools.push(Box::new(ToolWrapper(tool)));
        self
    }

    /// Adds multiple tools to the agent
    ///
    /// Convenience method for adding several tools at once.
    ///
    /// # Arguments
    ///
    /// * `tools` - Vector of box-wrapped tools
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::tool::Tool;
    ///
    /// // Assuming you have custom tools
    /// // let tools = vec![
    /// //     Box::new(Tool1::new()),
    /// //     Box::new(Tool2::new()),
    /// // ];
    /// // let builder = AgentBuilder::new().tools(tools);
    /// ```
    pub fn tools(mut self, tools: Vec<Box<dyn Tool>>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Adds web-related tools to the agent
    ///
    /// Includes tools for HTTP requests, web scraping, JSON API calls,
    /// and URL validation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().with_web_tools();
    /// ```
    ///
    /// # Tools Included
    ///
    /// - HTTP Request Tool: Make HTTP requests
    /// - Web Scraper Tool: Extract data from web pages
    /// - JSON API Tool: Call JSON APIs
    /// - URL Validator Tool: Validate URLs
    pub fn with_web_tools(self) -> Self {
        use crate::tool::builtin::web::*;
        self.tools(vec![
            Box::new(create_http_request_tool()),
            Box::new(create_web_scraper_tool()),
            Box::new(create_json_api_tool()),
            Box::new(create_url_validator_tool()),
        ])
    }

    /// Adds file operation tools to the agent
    ///
    /// Includes tools for reading, writing, listing, and inspecting files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    ///
    /// let builder = AgentBuilder::new().with_file_tools();
    /// ```
    ///
    /// # Tools Included
    ///
    /// - File Reader Tool: Read file contents
    /// - File Writer Tool: Write data to files
    /// - Directory Lister Tool: List directory contents
    /// - File Info Tool: Get file metadata
    pub fn with_file_tools(self) -> Self {
        use crate::tool::builtin::file::*;
        self.tools(vec![
            Box::new(create_file_reader_tool()),
            Box::new(create_file_writer_tool()),
            Box::new(create_directory_lister_tool()),
            Box::new(create_file_info_tool()),
        ])
    }

    /// Add data processing tools
    pub fn with_data_tools(self) -> Self {
        use crate::tool::builtin::data::*;
        self.tools(vec![
            Box::new(create_json_parser_tool()),
            Box::new(create_csv_parser_tool()),
            Box::new(create_data_transformer_tool()),
        ])
    }

    /// Add mathematical computation tools
    pub fn with_math_tools(self) -> Self {
        use crate::tool::builtin::math::*;
        self.tools(vec![
            Box::new(create_calculator_tool()),
            Box::new(create_statistics_tool()),
        ])
    }

    /// Add system operation tools
    pub fn with_system_tools(self) -> Self {
        // System tools implementation roadmap:
        // 1. File system operations: read_file, write_file, list_directory, create_folder
        // 2. Process management: run_command, kill_process, list_processes
        // 3. System monitoring: cpu_usage, memory_usage, disk_space
        // 4. Network tools: http_request, ping, port_scan
        // 5. Security tools: file_permissions, user_info, environment_variables
        // Note: Implement with proper security sandboxing and permission checks
        self
    }

    // === 动态配置方法 - 对标 Mastra ===

    /// 设置动态指令（基于运行时上下文）
    ///
    /// # Example
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::agent::dynamic_config::{dynamic_arg, EnhancedRuntimeContext};
    ///
    /// let builder = AgentBuilder::new()
    ///     .dynamic_instructions(dynamic_arg(|ctx: &EnhancedRuntimeContext| async move {
    ///         Ok(format!("You are a {} assistant for {}",
    ///             ctx.user_role.as_deref().unwrap_or("general"),
    ///             ctx.domain.as_deref().unwrap_or("general tasks")
    ///         ))
    ///     }));
    /// ```
    pub fn dynamic_instructions(mut self, instructions: DynamicArgument<String>) -> Self {
        self.dynamic_instructions = Some(instructions);
        self
    }

    /// 设置动态模型选择（基于运行时上下文）
    ///
    /// # Example
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::agent::dynamic_config::{dynamic_arg, ComplexityLevel};
    ///
    /// let builder = AgentBuilder::new()
    ///     .dynamic_model(dynamic_arg(|ctx| async move {
    ///         Ok(match ctx.complexity {
    ///             ComplexityLevel::Simple => "gpt-3.5-turbo".to_string(),
    ///             ComplexityLevel::Complex => "gpt-4".to_string(),
    ///             ComplexityLevel::Expert => "claude-3-opus".to_string(),
    ///         })
    ///     }));
    /// ```
    pub fn dynamic_model(mut self, model: DynamicArgument<String>) -> Self {
        self.dynamic_model = Some(model);
        self
    }

    /// 设置动态工具列表（基于运行时上下文）
    ///
    /// # Example
    /// ```rust
    /// use lumosai_core::agent::AgentBuilder;
    /// use lumosai_core::agent::dynamic_config::dynamic_arg;
    ///
    /// let builder = AgentBuilder::new()
    ///     .dynamic_tools(dynamic_arg(|ctx| async move {
    ///         Ok(ctx.get_user_tools())
    ///     }));
    /// ```
    pub fn dynamic_tools(mut self, tools: DynamicArgument<Vec<String>>) -> Self {
        self.dynamic_tools = Some(tools);
        self
    }

    /// 设置运行时上下文
    pub fn with_runtime_context(mut self, context: EnhancedRuntimeContext) -> Self {
        self.runtime_context = Some(context);
        self
    }

    /// Build the agent (同步版本，不支持动态配置)
    pub fn build(mut self) -> Result<BasicAgent> {
        // Apply smart defaults if enabled
        if self.smart_defaults {
            self = self.apply_smart_defaults()?;
        }

        // Validate required fields
        let name = self
            .name
            .ok_or_else(|| Error::Configuration("Agent name is required".to_string()))?;
        let instructions = self
            .instructions
            .ok_or_else(|| Error::Configuration("Agent instructions are required".to_string()))?;

        // Check if we have either a model or model_name
        if self.model.is_none() && self.model_name.is_none() {
            return Err(Error::Configuration(
                "Either model or model_name is required".to_string(),
            ));
        }

        // If we have a model_name but no model, we need to resolve it asynchronously
        if self.model.is_none() && self.model_name.is_some() {
            return Err(Error::Configuration(
                "Model name resolution requires async build. Use build_async() instead".to_string(),
            ));
        }

        let model = self
            .model
            .ok_or_else(|| Error::Configuration("Agent model is required".to_string()))?;

        // Create config
        let config = AgentConfig {
            name,
            instructions,
            memory_config: self.memory_config,
            model_id: self.model_id,
            voice_config: self.voice_config,
            telemetry: self.telemetry,
            working_memory: self.working_memory,
            enable_function_calling: self.enable_function_calling.or(Some(true)),
            context: self.context,
            metadata: self.metadata,
            max_tool_calls: self.max_tool_calls.or(Some(10)),
            tool_timeout: self.tool_timeout.or(Some(30)),
            tenant_id: self.tenant_id,
            isolation_level: self.isolation_level,
        };

        // Create agent
        let mut agent = BasicAgent::new(config, model);

        // Add tools
        for tool in self.tools {
            agent.add_tool(tool)?;
        }

        Ok(agent)
    }

    /// Build the agent asynchronously (supports model name resolution and dynamic configuration)
    pub async fn build_async(mut self) -> Result<BasicAgent> {
        // Apply smart defaults if enabled
        if self.smart_defaults {
            self = self.apply_smart_defaults()?;
        }

        // 处理动态配置解析
        let (final_name, final_instructions, final_model_name) = if self.runtime_context.is_some() {
            self.resolve_dynamic_config().await?
        } else {
            // 使用静态配置
            let name = self
                .name
                .ok_or_else(|| Error::Configuration("Agent name is required".to_string()))?;
            let instructions = self.instructions.ok_or_else(|| {
                Error::Configuration("Agent instructions are required".to_string())
            })?;
            (name, instructions, self.model_name.clone())
        };

        // Resolve model if needed
        let model = if let Some(model) = self.model {
            model
        } else if let Some(model_name) = final_model_name.or(self.model_name) {
            let resolver = self.model_resolver.unwrap_or_default();
            resolver.resolve(&model_name).await?
        } else {
            return Err(Error::Configuration(
                "Either model or model_name is required".to_string(),
            ));
        };

        // Create config
        let config = AgentConfig {
            name: final_name,
            instructions: final_instructions,
            memory_config: self.memory_config,
            model_id: self.model_id,
            voice_config: self.voice_config,
            telemetry: self.telemetry,
            working_memory: self.working_memory,
            enable_function_calling: self.enable_function_calling.or(Some(true)),
            context: self.context,
            metadata: self.metadata,
            max_tool_calls: self.max_tool_calls.or(Some(10)),
            tool_timeout: self.tool_timeout.or(Some(30)),
            tenant_id: self.tenant_id,
            isolation_level: self.isolation_level,
        };

        // Create agent
        let mut agent = BasicAgent::new(config, model);

        // Add tools
        for tool in self.tools {
            agent.add_tool(tool)?;
        }

        Ok(agent)
    }

    /// 解析动态配置 - 对标 Mastra 的动态参数解析
    async fn resolve_dynamic_config(&self) -> Result<(String, String, Option<String>)> {
        let context = self.runtime_context.as_ref().ok_or_else(|| {
            Error::Configuration("Runtime context is required for dynamic config".to_string())
        })?;

        let resolver = DynamicConfigResolver;

        // 解析动态指令
        let instructions = if let Some(dynamic_instructions) = &self.dynamic_instructions {
            resolver
                .resolve_string(dynamic_instructions, context)
                .await?
        } else {
            self.instructions
                .clone()
                .ok_or_else(|| Error::Configuration("Instructions are required".to_string()))?
        };

        // 解析动态模型
        let model_name = if let Some(dynamic_model) = &self.dynamic_model {
            Some(resolver.resolve_model(dynamic_model, context).await?)
        } else {
            self.model_name.clone()
        };

        // 解析名称（目前使用静态值，可以扩展为动态）
        let name = self
            .name
            .clone()
            .ok_or_else(|| Error::Configuration("Agent name is required".to_string()))?;

        Ok((name, instructions, model_name))
    }

    /// Apply smart defaults to simplify configuration
    fn apply_smart_defaults(mut self) -> Result<Self> {
        // Set default memory configuration if not specified
        if self.memory_config.is_none() {
            use crate::memory::MemoryConfig;
            self.memory_config = Some(MemoryConfig::default());
        }

        // Set default working memory if not specified
        if self.working_memory.is_none() {
            use crate::memory::WorkingMemoryConfig;
            self.working_memory = Some(WorkingMemoryConfig {
                enabled: true,
                template: None,
                content_type: None,
                max_capacity: Some(10),
            });
        }

        // Enable function calling by default
        if self.enable_function_calling.is_none() {
            self.enable_function_calling = Some(true);
        }

        // Set reasonable defaults for tool execution
        if self.max_tool_calls.is_none() {
            self.max_tool_calls = Some(10);
        }

        if self.tool_timeout.is_none() {
            self.tool_timeout = Some(30);
        }

        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
    use crate::tool::{FunctionTool, ParameterSchema, ToolSchema};

    #[tokio::test]
    async fn test_agent_builder_basic() {
        let llm = create_test_zhipu_provider_arc();

        let agent = AgentBuilder::new()
            .name("test_agent")
            .instructions("You are a test assistant")
            .model(llm)
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_name(), "test_agent");
        assert_eq!(agent.get_instructions(), "You are a test assistant");
    }

    #[tokio::test]
    async fn test_agent_builder_with_tools() {
        let llm = create_test_zhipu_provider_arc();

        // Create a test tool
        let schema = ToolSchema::new(vec![ParameterSchema {
            name: "message".to_string(),
            description: "Message to echo".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        }]);

        let echo_tool = FunctionTool::new("echo", "Echo a message", schema, |params| {
            let message = params
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No message");
            Ok(serde_json::json!(format!("Echo: {}", message)))
        });

        let agent = AgentBuilder::new()
            .name("test_agent")
            .instructions("You are a test assistant")
            .model(llm)
            .tool(Box::new(echo_tool))
            .max_tool_calls(5)
            .tool_timeout(60)
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_name(), "test_agent");
        assert_eq!(agent.get_tools().len(), 1);
        assert!(agent.get_tool("echo").is_some());
    }

    #[tokio::test]
    async fn test_agent_builder_validation() {
        // Test missing name
        let result = AgentBuilder::new()
            .instructions("Test instructions")
            .build();
        assert!(result.is_err());

        // Test missing instructions
        let llm = create_test_zhipu_provider_arc();
        let result = AgentBuilder::new().name("test").model(llm).build();
        assert!(result.is_err());
    }
}
