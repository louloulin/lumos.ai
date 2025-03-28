//! Agent模块提供了执行工具并生成回应的AI Agent功能

mod config;
mod executor;

pub use config::{AgentConfig, AgentGenerateOptions};
pub use executor::{Agent, ToolCall};

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use serde_json::Value;
    
    use crate::llm::{LlmOptions, LlmProvider, Message};
    use crate::tool::{FunctionTool, ParameterSchema, ToolSchema};
    use crate::Result;
    use super::*;
    
    struct MockLlmProvider {
        responses: Vec<String>,
    }
    
    impl MockLlmProvider {
        fn new(responses: Vec<String>) -> Self {
            Self { responses }
        }
    }
    
    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
            // Get the appropriate response based on the current state
            let messages = _options.messages.as_ref().unwrap_or(&vec![]);
            let index = messages.len().saturating_sub(2).min(self.responses.len() - 1);
            Ok(self.responses[index].clone())
        }
        
        async fn generate_stream<'a>(
            &'a self,
            _prompt: &'a str,
            _options: &'a LlmOptions,
        ) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
            unimplemented!("Streaming not implemented for mock provider")
        }
        
        async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
            unimplemented!("Embeddings not implemented for mock provider")
        }
    }
    
    #[tokio::test]
    async fn test_agent_with_tool() {
        // Create a mock LLM provider that first calls a tool, then provides a final response
        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "Using the tool 'echo' with parameters: { \"message\": \"Hello from tool!\" }".to_string(),
            "The tool returned: Echo: Hello from tool!".to_string(),
        ]));
        
        // Create an echo tool
        let schema = ToolSchema {
            parameters: vec![
                ParameterSchema {
                    name: "message".to_string(),
                    description: "The message to echo".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                },
            ],
        };
        
        let echo_tool = FunctionTool::new(
            "echo".to_string(),
            "Echoes the input message".to_string(),
            schema,
            |params| {
                let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
                Ok(Value::String(format!("Echo: {}", message)))
            },
        );
        
        // Create an agent
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a test agent.".to_string(),
            memory_config: None,
        };
        
        let mut agent = Agent::new(config, mock_llm);
        agent.add_tool(Arc::new(echo_tool));
        
        // Generate a response
        let response = agent.generate("Hello", &AgentGenerateOptions::default()).await.unwrap();
        
        assert_eq!(response, "The tool returned: Echo: Hello from tool!");
    }
} 