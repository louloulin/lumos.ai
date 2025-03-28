//! 智能体模块，负责处理用户请求并调用工具
//!
//! 智能体（Agent）是Lomusai的核心组件之一，用于响应用户查询、调用工具，并提供智能化的交互体验。
//! 智能体利用大语言模型和工具功能完成复杂任务。

mod config;
mod types;
mod trait_def;
mod executor;

pub use config::AgentConfig;
pub use trait_def::Agent;
pub use executor::BasicAgent;
pub use types::*;

/// 创建基本智能体
/// 
/// # Arguments
/// 
/// * `config` - 智能体配置
/// * `llm` - LLM提供者
/// 
/// # Returns
/// 
/// 返回一个新的BasicAgent实例
pub fn create_basic_agent<P: crate::llm::LlmProvider + 'static>(
    config: AgentConfig, 
    llm: std::sync::Arc<P>
) -> BasicAgent {
    BasicAgent::new(config, llm)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use async_trait::async_trait;
    
    use crate::llm::{LlmOptions, LlmProvider, Message, Role};
    use crate::tool::{FunctionTool, ParameterSchema, ToolSchema, ToolExecutionOptions};
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
                Ok(serde_json::json!(format!("Echo: {}", message)))
            },
        );
        
        // Create an agent
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a test agent.".to_string(),
            memory_config: None,
        };
        
        let mut agent = create_basic_agent(config, mock_llm);
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