//! Agent builder for simplified agent creation
//! 
//! This module provides a fluent builder API for creating agents,
//! inspired by Mastra's design but optimized for Rust.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

use crate::{Result, Error};
use crate::llm::LlmProvider;
use crate::tool::Tool;
use crate::memory::{MemoryConfig, WorkingMemoryConfig};
use super::{AgentConfig, BasicAgent};
use super::trait_def::Agent;
use super::types::{VoiceConfig, TelemetrySettings};

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
    memory_config: Option<MemoryConfig>,
    model_id: Option<String>,
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
            memory_config: None,
            model_id: None,
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
        }
    }

    /// Enable smart defaults for easier configuration
    pub fn enable_smart_defaults(mut self) -> Self {
        self.smart_defaults = true;
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

    /// Set the model ID
    pub fn model_id<S: Into<String>>(mut self, model_id: S) -> Self {
        self.model_id = Some(model_id.into());
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
        // TODO: Implement system tools when available
        self
    }







    /// Build the agent
    pub fn build(mut self) -> Result<BasicAgent> {
        // Apply smart defaults if enabled
        if self.smart_defaults {
            self = self.apply_smart_defaults()?;
        }

        // Validate required fields
        let name = self.name.ok_or_else(|| Error::Configuration("Agent name is required".to_string()))?;
        let instructions = self.instructions.ok_or_else(|| Error::Configuration("Agent instructions are required".to_string()))?;
        let model = self.model.ok_or_else(|| Error::Configuration("Agent model is required".to_string()))?;

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
        };

        // Create agent
        let mut agent = BasicAgent::new(config, model);

        // Add tools
        for tool in self.tools {
            agent.add_tool(tool)?;
        }

        Ok(agent)
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
    use crate::tool::{FunctionTool, ToolSchema, ParameterSchema};

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
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "message".to_string(),
                description: "Message to echo".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
        ]);
        
        let echo_tool = FunctionTool::new(
            "echo",
            "Echo a message",
            schema,
            |params| {
                let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
                Ok(serde_json::json!(format!("Echo: {}", message)))
            },
        );

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
        let result = AgentBuilder::new()
            .name("test")
            .model(llm)
            .build();
        assert!(result.is_err());
    }
}
