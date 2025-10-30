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

/// Wrapper to convert Arc<dyn Tool> to Box<dyn Tool>
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
/// # Example
///
/// ```rust
/// use lumosai_core::agent::AgentBuilder;
/// use lumosai_core::llm::MockLlmProvider;
/// use std::sync::Arc;
///
/// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
///
/// let agent = AgentBuilder::new()
///     .name("assistant")
///     .instructions("You are a helpful assistant")
///     .model(llm)
///     .max_tool_calls(5)
///     .build()
///     .expect("Failed to build agent");
/// ```
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
    /// Create a new agent builder
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

    /// Enable smart defaults for easier configuration
    pub fn enable_smart_defaults(mut self) -> Self {
        self.smart_defaults = true;
        self
    }

    /// Set tenant ID for multi-tenant support
    pub fn tenant_id<S: Into<String>>(mut self, tenant_id: S) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Set isolation level for multi-tenancy
    pub fn isolation_level<S: Into<String>>(mut self, level: S) -> Self {
        self.isolation_level = Some(level.into());
        self
    }

    /// Set the agent name
    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the agent instructions
    pub fn instructions<S: Into<String>>(mut self, instructions: S) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Set the LLM model provider
    pub fn model(mut self, model: Arc<dyn LlmProvider>) -> Self {
        self.model = Some(model);
        self
    }

    /// Set the model using a string name (e.g., "gpt-4", "claude-3-sonnet")
    /// This will automatically resolve the model name to the appropriate provider
    pub fn model_name<S: Into<String>>(mut self, model_name: S) -> Self {
        self.model_name = Some(model_name.into());
        if self.model_resolver.is_none() {
            self.model_resolver = Some(ModelResolver::new());
        }
        self
    }

    /// Set the model ID
    pub fn model_id<S: Into<String>>(mut self, model_id: S) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    /// Set the temperature for the model
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the maximum number of tokens for the model
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set memory configuration
    pub fn memory_config(mut self, config: MemoryConfig) -> Self {
        self.memory_config = Some(config);
        self
    }

    /// Set voice configuration
    pub fn voice_config(mut self, config: VoiceConfig) -> Self {
        self.voice_config = Some(config);
        self
    }

    /// Set telemetry settings
    pub fn telemetry(mut self, telemetry: TelemetrySettings) -> Self {
        self.telemetry = Some(telemetry);
        self
    }

    /// Set working memory configuration
    pub fn working_memory(mut self, config: WorkingMemoryConfig) -> Self {
        self.working_memory = Some(config);
        self
    }

    /// Enable or disable function calling
    pub fn enable_function_calling(mut self, enabled: bool) -> Self {
        self.enable_function_calling = Some(enabled);
        self
    }

    /// Add context data
    pub fn context(mut self, context: HashMap<String, Value>) -> Self {
        self.context = Some(context);
        self
    }

    /// Add a single context value
    pub fn add_context<K: Into<String>>(mut self, key: K, value: Value) -> Self {
        if self.context.is_none() {
            self.context = Some(HashMap::new());
        }
        if let Some(ref mut context) = self.context {
            context.insert(key.into(), value);
        }
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add a single metadata value
    pub fn add_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        if self.metadata.is_none() {
            self.metadata = Some(HashMap::new());
        }
        if let Some(ref mut metadata) = self.metadata {
            metadata.insert(key.into(), value.into());
        }
        self
    }

    /// Set maximum number of tool calls
    pub fn max_tool_calls(mut self, max: u32) -> Self {
        self.max_tool_calls = Some(max);
        self
    }

    /// Set tool execution timeout in seconds
    pub fn tool_timeout(mut self, timeout: u64) -> Self {
        self.tool_timeout = Some(timeout);
        self
    }

    /// Add a tool to the agent
    pub fn tool(mut self, tool: Box<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add a tool to the agent (Arc version)
    pub fn add_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        // Convert Arc to Box by cloning the tool
        // Note: This requires the Tool trait to implement Clone or we need a different approach
        // For now, we'll use a workaround
        self.tools.push(Box::new(ToolWrapper(tool)));
        self
    }

    /// Add multiple tools to the agent
    pub fn tools(mut self, tools: Vec<Box<dyn Tool>>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Add tools from a tool collection
    pub fn with_web_tools(self) -> Self {
        use crate::tool::builtin::web::*;
        self.tools(vec![
            Box::new(create_http_request_tool()),
            Box::new(create_web_scraper_tool()),
            Box::new(create_json_api_tool()),
            Box::new(create_url_validator_tool()),
        ])
    }

    /// Add file operation tools
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
    use crate::llm::MockLlmProvider;
    use crate::tool::{FunctionTool, ParameterSchema, ToolSchema};

    #[tokio::test]
    async fn test_agent_builder_basic() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

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
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

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
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        let result = AgentBuilder::new().name("test").model(llm).build();
        assert!(result.is_err());
    }
}
