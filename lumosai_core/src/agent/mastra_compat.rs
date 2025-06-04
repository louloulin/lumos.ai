//! Mastra-compatible API layer
//! 
//! This module provides a Mastra-compatible API for easy migration and familiar usage patterns

use crate::agent::{AgentBuilder, BasicAgent};
use crate::agent::trait_def::Agent as AgentTrait;
use crate::llm::LlmProvider;
use crate::tool::Tool;
use crate::tool::builtin::{create_all_builtin_tools, BuiltinToolsConfig};
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;

/// Mastra-style Agent creation API
/// Provides a familiar interface for developers coming from Mastra
pub struct Agent;

impl Agent {
    /// Create a new agent with Mastra-style API
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use lumosai_core::agent::mastra_compat::Agent;
    /// use lumosai_core::llm::MockLlmProvider;
    /// use std::sync::Arc;
    /// 
    /// let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    /// 
    /// let agent = Agent::create()
    ///     .name("assistant")
    ///     .instructions("You are a helpful assistant")
    ///     .model(llm)
    ///     .build()
    ///     .expect("Failed to create agent");
    /// ```
    pub fn create() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Create an agent with built-in tools
    /// Similar to Mastra's tool integration
    pub fn with_tools() -> AgentBuilderWithTools {
        AgentBuilderWithTools::new()
    }

    /// Create an agent from a configuration object
    /// Similar to Mastra's configuration-based creation
    pub fn from_config(config: AgentConfig) -> crate::Result<BasicAgent> {
        let mut builder = AgentBuilder::new()
            .name(config.name)
            .instructions(config.instructions);

        if let Some(model) = config.model {
            builder = builder.model(model);
        }

        if let Some(memory) = config.enable_memory {
            if memory {
                // Create a basic working memory config
                let working_memory_config = crate::memory::WorkingMemoryConfig {
                    enabled: true,
                    template: None,
                    content_type: None,
                    max_capacity: Some(100),
                };
                builder = builder.working_memory(working_memory_config);
            }
        }

        if let Some(max_calls) = config.max_tool_calls {
            builder = builder.max_tool_calls(max_calls);
        }

        if let Some(timeout) = config.tool_timeout {
            builder = builder.tool_timeout(timeout);
        }

        let mut agent = builder.build()?;

        // Add tools if specified
        if let Some(tools) = config.tools {
            for tool in tools {
                AgentTrait::add_tool(&mut agent, tool)?;
            }
        }

        Ok(agent)
    }
}

/// Agent configuration structure
/// Similar to Mastra's agent configuration
pub struct AgentConfig {
    pub name: String,
    pub instructions: String,
    pub model: Option<Arc<dyn LlmProvider>>,
    pub tools: Option<Vec<Box<dyn Tool>>>,
    pub enable_memory: Option<bool>,
    pub max_tool_calls: Option<u32>,
    pub tool_timeout: Option<u64>,
    pub metadata: Option<HashMap<String, Value>>,
}

impl AgentConfig {
    pub fn new(name: &str, instructions: &str) -> Self {
        Self {
            name: name.to_string(),
            instructions: instructions.to_string(),
            model: None,
            tools: None,
            enable_memory: None,
            max_tool_calls: None,
            tool_timeout: None,
            metadata: None,
        }
    }

    pub fn with_model(mut self, model: Arc<dyn LlmProvider>) -> Self {
        self.model = Some(model);
        self
    }

    pub fn with_tools(mut self, tools: Vec<Box<dyn Tool>>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_memory(mut self, enabled: bool) -> Self {
        self.enable_memory = Some(enabled);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Agent builder with built-in tools support
/// Provides easy access to common tools like Mastra
pub struct AgentBuilderWithTools {
    builder: AgentBuilder,
    selected_tools: Vec<String>,
}

impl AgentBuilderWithTools {
    pub fn new() -> Self {
        Self {
            builder: AgentBuilder::new(),
            selected_tools: Vec::new(),
        }
    }

    pub fn name<S: Into<String>>(mut self, name: S) -> Self {
        self.builder = self.builder.name(name);
        self
    }

    pub fn instructions<S: Into<String>>(mut self, instructions: S) -> Self {
        self.builder = self.builder.instructions(instructions);
        self
    }

    pub fn model(mut self, model: Arc<dyn LlmProvider>) -> Self {
        self.builder = self.builder.model(model);
        self
    }

    /// Add web tools (HTTP, scraping, etc.)
    pub fn with_web_tools(mut self) -> Self {
        self.selected_tools.extend(vec![
            "http_request".to_string(),
            "web_scraper".to_string(),
            "json_api".to_string(),
            "url_validator".to_string(),
        ]);
        self
    }

    /// Add file tools (read, write, list)
    pub fn with_file_tools(mut self) -> Self {
        self.selected_tools.extend(vec![
            "file_reader".to_string(),
            "file_writer".to_string(),
            "directory_lister".to_string(),
            "file_info".to_string(),
        ]);
        self
    }

    /// Add data processing tools (JSON, CSV, etc.)
    pub fn with_data_tools(mut self) -> Self {
        self.selected_tools.extend(vec![
            "json_parser".to_string(),
            "csv_parser".to_string(),
            "data_transformer".to_string(),
        ]);
        self
    }

    /// Add system tools (datetime, UUID, etc.)
    pub fn with_system_tools(mut self) -> Self {
        self.selected_tools.extend(vec![
            "datetime".to_string(),
            "uuid_generator".to_string(),
            "hash_generator".to_string(),
        ]);
        self
    }

    /// Add math tools (calculator, statistics)
    pub fn with_math_tools(mut self) -> Self {
        self.selected_tools.extend(vec![
            "calculator".to_string(),
            "statistics".to_string(),
        ]);
        self
    }

    /// Add all available tools
    pub fn with_all_tools(self) -> Self {
        self.with_web_tools()
            .with_file_tools()
            .with_data_tools()
            .with_system_tools()
            .with_math_tools()
    }

    /// Build the agent with selected tools
    pub fn build(mut self) -> crate::Result<BasicAgent> {
        // Get all available tools
        let config = BuiltinToolsConfig::default();
        let all_tools = create_all_builtin_tools(&config);
        
        // Filter tools based on selection
        for tool in all_tools {
            if let Some(name) = tool.name() {
                if self.selected_tools.contains(&name.to_string()) {
                    self.builder = self.builder.tool(tool);
                }
            }
        }

        self.builder.build()
    }
}

/// Utility functions for Mastra compatibility
pub mod utils {
    use super::*;

    /// Create a quick agent for simple tasks
    /// Similar to Mastra's quick setup
    pub fn quick_agent(
        name: &str,
        instructions: &str,
        model: Arc<dyn LlmProvider>,
    ) -> crate::Result<BasicAgent> {
        Agent::create()
            .name(name)
            .instructions(instructions)
            .model(model)
            .build()
    }

    /// Create an agent with common tools
    /// Similar to Mastra's preset configurations
    pub fn agent_with_common_tools(
        name: &str,
        instructions: &str,
        model: Arc<dyn LlmProvider>,
    ) -> crate::Result<BasicAgent> {
        Agent::with_tools()
            .name(name)
            .instructions(instructions)
            .model(model)
            .with_data_tools()
            .with_system_tools()
            .with_math_tools()
            .build()
    }

    /// Create a web-enabled agent
    /// For agents that need web access
    pub fn web_agent(
        name: &str,
        instructions: &str,
        model: Arc<dyn LlmProvider>,
    ) -> crate::Result<BasicAgent> {
        Agent::with_tools()
            .name(name)
            .instructions(instructions)
            .model(model)
            .with_web_tools()
            .with_data_tools()
            .build()
    }

    /// Create a file processing agent
    /// For agents that work with files
    pub fn file_agent(
        name: &str,
        instructions: &str,
        model: Arc<dyn LlmProvider>,
    ) -> crate::Result<BasicAgent> {
        Agent::with_tools()
            .name(name)
            .instructions(instructions)
            .model(model)
            .with_file_tools()
            .with_data_tools()
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockLlmProvider;

    #[tokio::test]
    async fn test_mastra_style_agent_creation() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = Agent::create()
            .name("test_agent")
            .instructions("You are a test assistant")
            .model(llm)
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "test_agent");
        assert_eq!(agent.get_instructions(), "You are a test assistant");
    }

    #[tokio::test]
    async fn test_agent_with_tools() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = Agent::with_tools()
            .name("tool_agent")
            .instructions("You are a tool-enabled assistant")
            .model(llm)
            .with_math_tools()
            .with_data_tools()
            .build()
            .expect("Failed to create agent");

        assert_eq!(agent.get_name(), "tool_agent");
        assert!(agent.get_tools().len() > 0);
    }

    #[tokio::test]
    async fn test_quick_agent() {
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let agent = utils::quick_agent(
            "quick_test",
            "Quick test agent",
            llm,
        ).expect("Failed to create quick agent");

        assert_eq!(agent.get_name(), "quick_test");
    }
}
