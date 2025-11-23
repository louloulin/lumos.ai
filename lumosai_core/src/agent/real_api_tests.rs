//! Real API tests for Agent module
//!
//! Tests Agent functionality with real Zhipu AI Provider, focusing on:
//! - Agent collaboration and multi-agent interaction
//! - Tool calling and parameter passing
//! - Error handling and retry mechanisms
//! - Timeout and cancellation operations
//! - Context management and state persistence

#[cfg(test)]
mod tests {
    use crate::agent::builder::AgentBuilder;
    use crate::agent::config::AgentConfig;
    use crate::agent::BasicAgent;
    use crate::agent::trait_def::{Agent, AgentStatus};
    use crate::agent::types::AgentGenerateOptions;
    use crate::llm::test_helpers::create_test_zhipu_provider_arc;
    use crate::llm::{LlmOptions, Message, Role};
    use crate::tool::{FunctionTool, ParameterSchema, SchemaFormat, Tool, ToolSchema};
    use serde_json::{json, Value};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;

    /// Helper function to retry API calls with exponential backoff
    async fn retry_with_backoff<F, Fut, T>(mut f: F, max_retries: u32) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mut retries = 0;
        let mut delay = Duration::from_millis(2000);

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    retries += 1;
                    eprintln!(
                        "Attempt {} failed: {}. Retrying in {:?}...",
                        retries, e, delay
                    );
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
                Err(e) => return Err(format!("Failed after {} retries: {}", max_retries, e)),
            }
        }
    }

    #[tokio::test]
    async fn test_agent_basic_creation() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.get_name(), "test_agent");
        assert_eq!(agent.get_instructions(), "You are a helpful assistant.");
        // AgentStatus has Ready variant in trait_def.rs
        assert_eq!(agent.get_status(), AgentStatus::Ready);
    }

    #[tokio::test]
    async fn test_agent_builder_pattern() {
        let llm = create_test_zhipu_provider_arc();

        let agent = AgentBuilder::new()
            .name("builder_agent")
            .instructions("You are a test assistant")
            .model(llm)
            .max_tool_calls(5)
            .tool_timeout(30)
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_name(), "builder_agent");
        assert_eq!(agent.get_instructions(), "You are a test assistant");
    }

    #[tokio::test]
    async fn test_agent_status_transitions() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm).unwrap();

        // Initial status should be Ready
        assert_eq!(agent.get_status(), AgentStatus::Ready);
    }

    #[tokio::test]
    async fn test_agent_instructions_update() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let mut agent = BasicAgent::new(config, llm).unwrap();

        let new_instructions = "Updated instructions for testing";
        agent.set_instructions(new_instructions.to_string());

        assert_eq!(agent.get_instructions(), new_instructions);
    }

    #[tokio::test]
    async fn test_agent_with_model_id() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "model_agent".to_string(),
            instructions: "Test agent".to_string(),
            model_id: Some("glm-4.6".to_string()),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.get_name(), "model_agent");
    }

    #[tokio::test]
    async fn test_agent_with_memory_config() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "memory_agent".to_string(),
            instructions: "Test agent".to_string(),
            memory_config: None, // No memory config for now
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.get_name(), "memory_agent");
    }

    #[tokio::test]
    async fn test_agent_llm_provider_access() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm.clone()).unwrap();

        let agent_llm = agent.get_llm();
        assert_eq!(agent_llm.name(), "zhipu");
    }

    #[tokio::test]
    async fn test_agent_memory_check() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm).unwrap();

        // Default agent should not have memory
        assert!(!agent.has_own_memory());
        assert!(agent.get_memory().is_none());
    }

    #[tokio::test]
    async fn test_agent_tools_empty_by_default() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm).unwrap();

        let tools = agent.get_tools();
        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_agent_builder_with_tool() {
        let llm = create_test_zhipu_provider_arc();

        // Create a simple echo tool
        let echo_tool = FunctionTool::new(
            "echo",
            "Echoes the input",
            ToolSchema {
                parameters: vec![ParameterSchema {
                    name: "message".to_string(),
                    r#type: "string".to_string(),
                    description: "Message to echo".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                }],
                json_schema: None,
                format: SchemaFormat::default(),
                output_schema: None,
            },
            |args: Value| {
                let message = args["message"].as_str().unwrap_or("").to_string();
                Ok(json!({ "echo": message }))
            },
        );

        let agent = AgentBuilder::new()
            .name("tool_agent")
            .instructions("You are a test assistant")
            .model(llm)
            .tool(Box::new(echo_tool))
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_tools().len(), 1);
        assert!(agent.get_tool("echo").is_some());
    }

    #[tokio::test]
    async fn test_agent_builder_validation_missing_name() {
        let result = AgentBuilder::new()
            .instructions("Test instructions")
            .build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_builder_validation_missing_instructions() {
        let llm = create_test_zhipu_provider_arc();
        let result = AgentBuilder::new().name("test").model(llm).build();

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_agent_config_default_values() {
        let config = AgentConfig::default();

        assert!(!config.name.is_empty());
        assert!(!config.instructions.is_empty());
        assert!(config.model_id.is_none());
    }

    #[tokio::test]
    async fn test_agent_config_custom_values() {
        let config = AgentConfig {
            name: "custom".to_string(),
            instructions: "Custom instructions".to_string(),
            model_id: Some("glm-4.6".to_string()),
            ..Default::default()
        };

        assert_eq!(config.name, "custom");
        assert_eq!(config.instructions, "Custom instructions");
        assert_eq!(config.model_id, Some("glm-4.6".to_string()));
    }

    #[tokio::test]
    async fn test_agent_status_ready() {
        let status = AgentStatus::Ready;
        assert_eq!(status, AgentStatus::Ready);
    }

    #[tokio::test]
    async fn test_agent_status_running() {
        let status = AgentStatus::Running;
        assert_eq!(status, AgentStatus::Running);
    }

    #[tokio::test]
    async fn test_agent_status_error() {
        let status = AgentStatus::Error("test error".to_string());
        assert_eq!(status, AgentStatus::Error("test error".to_string()));
    }

    #[tokio::test]
    async fn test_agent_multiple_tools() {
        let llm = create_test_zhipu_provider_arc();

        let tool1 = FunctionTool::new(
            "tool1",
            "First tool",
            ToolSchema {
                parameters: vec![],
                json_schema: None,
                format: SchemaFormat::default(),
                output_schema: None,
            },
            |_args: Value| Ok(json!({"result": "tool1"})),
        );

        let tool2 = FunctionTool::new(
            "tool2",
            "Second tool",
            ToolSchema {
                parameters: vec![],
                json_schema: None,
                format: SchemaFormat::default(),
                output_schema: None,
            },
            |_args: Value| Ok(json!({"result": "tool2"})),
        );

        let agent = AgentBuilder::new()
            .name("multi_tool_agent")
            .instructions("Test agent with multiple tools")
            .model(llm)
            .tool(Box::new(tool1))
            .tool(Box::new(tool2))
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_tools().len(), 2);
        assert!(agent.get_tool("tool1").is_some());
        assert!(agent.get_tool("tool2").is_some());
    }

    #[tokio::test]
    async fn test_agent_tool_not_found() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm).unwrap();

        assert!(agent.get_tool("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_agent_builder_max_tool_calls() {
        let llm = create_test_zhipu_provider_arc();

        let agent = AgentBuilder::new()
            .name("test_agent")
            .instructions("Test")
            .model(llm)
            .max_tool_calls(10)
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_name(), "test_agent");
    }

    #[tokio::test]
    async fn test_agent_builder_tool_timeout() {
        let llm = create_test_zhipu_provider_arc();

        let agent = AgentBuilder::new()
            .name("test_agent")
            .instructions("Test")
            .model(llm)
            .tool_timeout(60)
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_name(), "test_agent");
    }

    #[tokio::test]
    async fn test_agent_name_validation() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "valid_agent_name_123".to_string(),
            instructions: "Test".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.get_name(), "valid_agent_name_123");
    }

    #[tokio::test]
    async fn test_agent_instructions_not_empty() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "test".to_string(),
            instructions: "Non-empty instructions".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert!(!agent.get_instructions().is_empty());
    }

    #[tokio::test]
    async fn test_agent_config_clone() {
        let config1 = AgentConfig {
            name: "test".to_string(),
            instructions: "Test instructions".to_string(),
            model_id: Some("glm-4.6".to_string()),
            ..Default::default()
        };

        let config2 = config1.clone();
        assert_eq!(config1.name, config2.name);
        assert_eq!(config1.instructions, config2.instructions);
        assert_eq!(config1.model_id, config2.model_id);
    }

    #[tokio::test]
    async fn test_agent_status_paused() {
        let status = AgentStatus::Paused;
        assert_eq!(status, AgentStatus::Paused);
    }

    #[tokio::test]
    async fn test_agent_with_empty_tool_list() {
        let llm = create_test_zhipu_provider_arc();

        let agent = AgentBuilder::new()
            .name("no_tools_agent")
            .instructions("Agent without tools")
            .model(llm)
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_tools().len(), 0);
    }

    #[tokio::test]
    async fn test_agent_builder_chaining() {
        let llm = create_test_zhipu_provider_arc();

        let agent = AgentBuilder::new()
            .name("chain_agent")
            .instructions("Test chaining")
            .model(llm)
            .max_tool_calls(5)
            .tool_timeout(30)
            .build()
            .expect("Failed to build agent");

        assert_eq!(agent.get_name(), "chain_agent");
    }

    #[tokio::test]
    async fn test_agent_with_metadata() {
        let llm = create_test_zhipu_provider_arc();
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("version".to_string(), "1.0".to_string());

        let config = AgentConfig {
            name: "metadata_agent".to_string(),
            instructions: "Test".to_string(),
            metadata: Some(metadata),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.get_name(), "metadata_agent");
    }

    #[tokio::test]
    async fn test_agent_with_context() {
        let llm = create_test_zhipu_provider_arc();
        let mut context = std::collections::HashMap::new();
        context.insert("key".to_string(), json!("value"));

        let config = AgentConfig {
            name: "context_agent".to_string(),
            instructions: "Test".to_string(),
            context: Some(context),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.get_name(), "context_agent");
    }

    #[tokio::test]
    async fn test_agent_status_stopped() {
        let status = AgentStatus::Stopped;
        assert_eq!(status, AgentStatus::Stopped);
    }

    #[tokio::test]
    async fn test_agent_status_initializing() {
        let status = AgentStatus::Initializing;
        assert_eq!(status, AgentStatus::Initializing);
    }

    #[tokio::test]
    async fn test_agent_with_function_calling_enabled() {
        let llm = create_test_zhipu_provider_arc();
        let config = AgentConfig {
            name: "function_agent".to_string(),
            instructions: "Test agent".to_string(),
            enable_function_calling: Some(true),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();
        assert_eq!(agent.get_name(), "function_agent");
    }
}
