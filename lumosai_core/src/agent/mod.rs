//! Agent module for LLM-based agents

pub mod config;
pub mod trait_def;
pub mod executor;
pub mod evaluation;
pub mod message_utils;
pub mod types;
pub mod streaming;
pub mod websocket;

#[cfg(feature = "demos")]
pub mod websocket_demo;
#[cfg(feature = "demos")]
pub mod enhanced_streaming_demo;

#[cfg(test)]
mod mastra_integration_test;

pub use config::AgentConfig;
pub use trait_def::Agent;
pub use executor::BasicAgent;
pub use message_utils::{system_message, user_message, assistant_message, tool_message};

// Re-export streaming types
pub use streaming::{
    AgentEvent, 
    MemoryOperation, 
    StreamingConfig, 
    StreamingAgent, 
    IntoStreaming
};

// Re-export WebSocket streaming types
pub use websocket::{
    WebSocketMessage,
    WebSocketConnection,
    WebSocketConfig,
    WebSocketManager,
    WebSocketStreamingAgent,
    IntoWebSocketStreaming
};

// Re-export agent types
pub use types::{
    AgentGenerateOptions,
    AgentStreamOptions,
    AgentGenerateResult,
    AgentStep,
    AgentToolCall,
    VoiceConfig,
    TelemetrySettings,
    RuntimeContext,
    DynamicArgument,
    ToolsInput,
    ToolsetsInput,
};

// Re-export evaluation types
pub use evaluation::{
    EvaluationMetric,
    EvaluationResult,
    RelevanceMetric,
    LengthMetric,
    CompositeMetric,
};

/// Create a basic agent with default configuration
pub fn create_basic_agent(
    name: impl Into<String>,
    instructions: impl Into<String>,
    llm: std::sync::Arc<dyn crate::llm::LlmProvider>,
) -> BasicAgent {
    let _config = AgentConfig {
        name: name.into(),
        instructions: instructions.into(),
        memory_config: None,
        model_id: None,
        voice_config: None,
        telemetry: None,
        working_memory: None,
        enable_function_calling: Some(true),
    };
    
    BasicAgent::new(_config, llm)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use async_trait::async_trait;
    
    use crate::llm::{LlmOptions, LlmProvider, Message, Role};
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
            let messages = _options
                .extra
                .get("messages")
                .and_then(|v| v.as_array())
                .map(|msgs| msgs.len().saturating_sub(2).min(self.responses.len() - 1))
                .unwrap_or(0);
            
            Ok(self.responses[messages].clone())
        }
        
        async fn generate_with_messages(&self, messages: &[Message], _options: &LlmOptions) -> Result<String> {
            // Get the appropriate response based on the messages length
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
        // Create an echo tool
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "message".to_string(),
                description: "The message to echo".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            },
        ]);
        
        let echo_tool = FunctionTool::new(
            "echo",
            "Echoes the input message",
            schema,
            |params| {
                let message = params.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
                Ok(serde_json::json!(format!("Echo: {}", message)))
            },
        );
        
        // Create an agent
        let _config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a test agent.".to_string(),
            memory_config: None,
            model_id: None,
            voice_config: None,
            telemetry: None,
            working_memory: None,
            enable_function_calling: Some(true),
        };
        
        let mock_llm = Arc::new(MockLlmProvider::new(vec![
            "I'll help you with that! Using the tool 'echo' with parameters: {\"message\": \"Hello from tool!\"}".to_string(),
            "The tool returned: Echo: Hello from tool!".to_string(),
        ]));
        
        let mut agent = create_basic_agent(
            "TestAgent".to_string(),
            "You are a test agent.".to_string(), 
            mock_llm
        );
        agent.add_tool(Box::new(echo_tool)).unwrap();
        
        // Generate a response
        let user_message = Message {
            role: Role::User,
            content: "Hello".to_string(),
            metadata: None,
            name: None,
        };
        
        let result = agent.generate(&[user_message], &AgentGenerateOptions::default()).await.unwrap();
        
        assert_eq!(result.response, "The tool returned: Echo: Hello from tool!");
    }
}