//! Integration tests for enhanced function calling feature

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::agent::{BasicAgent, AgentConfig, AgentGenerateOptions};
use crate::agent::types::ToolCall;
use crate::llm::mock::MockLlmProvider;
use crate::llm::function_calling::{FunctionDefinition, FunctionCall};
use crate::tool::TestTool;
use crate::error::Result;

// 添加这个函数到lumosai_core/tests/function_calling.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_function_calling() -> Result<()> {
        // Create mock LLM provider that supports function calling
        let mut llm = MockLlmProvider::new();
        llm.set_supports_function_calling(true);
        
        // Set up function calling response
        llm.add_function_calling_response(
            vec![FunctionCall {
                id: Some("call_123".to_string()),
                name: "calculator".to_string(),
                arguments: r#"{"expression": "2+2"}"#.to_string(),
            }],
            Some("Calculating 2+2..."),
        );
        
        // Create agent config with function calling enabled
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a helpful assistant that can use tools.".to_string(),
            enable_function_calling: Some(true),
            ..Default::default()
        };
        
        // Create agent
        let agent = BasicAgent::new(config, Arc::new(llm));
        
        // Add calculator tool
        let mut tools = HashMap::new();
        tools.insert("calculator".to_string(), Box::new(TestTool::new("calculator", "A calculator tool")));
        agent.set_tools(tools);
        
        // Execute test
        let result = agent.generate("Calculate 2+2", &AgentGenerateOptions::default()).await?;
        
        // Verify results
        assert_eq!(result.steps.len(), 1);  // Should have at least one step
        assert_eq!(result.steps[0].tool_calls.len(), 1);  // Should have exactly one tool call
        assert_eq!(result.steps[0].tool_calls[0].name, "calculator");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_enhanced_tool_parsing() -> Result<()> {
        // Create a mock LLM provider that doesn't support function calling
        let mut llm = MockLlmProvider::new();
        llm.set_supports_function_calling(false);
        
        // Set up responses for different formats
        
        // 1. Standard regex format
        llm.add_response("I'll help you with that. Using the tool 'calculator' with parameters: {\"expression\": \"2+2\"}");
        
        // 2. Function-like format
        llm.add_response("Let me calculate that. calculator(expression=\"2+2\")");
        
        // 3. JSON code block format
        llm.add_response("I'll use the calculator tool:\n```json\n{\"tool\": \"calculator\", \"parameters\": {\"expression\": \"2+2\"}}\n```");
        
        // Create agent config with function calling disabled to force regex parsing
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a helpful assistant that can use tools.".to_string(),
            enable_function_calling: Some(false),
            ..Default::default()
        };
        
        // Create agent
        let agent = BasicAgent::new(config, Arc::new(llm));
        
        // Add calculator tool
        let mut tools = HashMap::new();
        tools.insert("calculator".to_string(), Box::new(TestTool::new("calculator", "A calculator tool")));
        agent.set_tools(tools);
        
        // Test standard regex format
        let result1 = agent.generate("Calculate 2+2 using standard format", &AgentGenerateOptions::default()).await?;
        assert_eq!(result1.steps[0].tool_calls.len(), 1);
        assert_eq!(result1.steps[0].tool_calls[0].name, "calculator");
        
        // Test function-like format
        let result2 = agent.generate("Calculate 2+2 using function format", &AgentGenerateOptions::default()).await?;
        assert_eq!(result2.steps[0].tool_calls.len(), 1);
        assert_eq!(result2.steps[0].tool_calls[0].name, "calculator");
        
        // Test JSON code block format
        let result3 = agent.generate("Calculate 2+2 using JSON format", &AgentGenerateOptions::default()).await?;
        assert_eq!(result3.steps[0].tool_calls.len(), 1);
        assert_eq!(result3.steps[0].tool_calls[0].name, "calculator");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_system_message_generation() -> Result<()> {
        // Test with function calling
        {
            let mut llm = MockLlmProvider::new();
            llm.set_supports_function_calling(true);
            
            let config = AgentConfig {
                name: "TestAgent".to_string(),
                instructions: "You are a helpful assistant.".to_string(),
                enable_function_calling: Some(true),
                ..Default::default()
            };
            
            let agent = BasicAgent::new(config, Arc::new(llm));
            
            // Add a tool
            let mut tools = HashMap::new();
            tools.insert("calculator".to_string(), Box::new(TestTool::new("calculator", "A calculator tool")));
            agent.set_tools(tools);
            
            // Capture the system message
            llm = MockLlmProvider::new();
            llm.set_supports_function_calling(true);
            llm.add_response("Test response");
            
            let agent = BasicAgent::new(
                AgentConfig {
                    name: "TestAgent".to_string(),
                    instructions: "You are a helpful assistant.".to_string(),
                    enable_function_calling: Some(true),
                    ..Default::default()
                },
                Arc::new(llm.clone())
            );
            
            // Add tools
            let mut tools = HashMap::new();
            tools.insert("calculator".to_string(), Box::new(TestTool::new("calculator", "A calculator tool")));
            agent.set_tools(tools);
            
            // Generate to capture the system message
            let _ = agent.generate("Test", &AgentGenerateOptions::default()).await?;
            
            // Get the captured messages
            let messages = llm.get_messages();
            assert!(!messages.is_empty());
            
            // Check system message for function calling
            let system_message = messages[0].content.clone();
            assert!(system_message.contains("You have access to specialized tools"));
            assert!(!system_message.contains("Using the tool"));
        }
        
        // Test without function calling
        {
            let mut llm = MockLlmProvider::new();
            llm.set_supports_function_calling(false);
            llm.add_response("Test response");
            
            let config = AgentConfig {
                name: "TestAgent".to_string(),
                instructions: "You are a helpful assistant.".to_string(),
                enable_function_calling: Some(false),
                ..Default::default()
            };
            
            let agent = BasicAgent::new(config, Arc::new(llm.clone()));
            
            // Add a tool
            let mut tools = HashMap::new();
            tools.insert("calculator".to_string(), Box::new(TestTool::new("calculator", "A calculator tool")));
            agent.set_tools(tools);
            
            // Generate to capture the system message
            let _ = agent.generate("Test", &AgentGenerateOptions::default()).await?;
            
            // Get the captured messages  
            let messages = llm.get_messages();
            assert!(!messages.is_empty());
            
            // Check system message for regex format
            let system_message = messages[0].content.clone();
            assert!(system_message.contains("To use a tool, use exactly the following format"));
            assert!(system_message.contains("calculator"));
        }
        
        Ok(())
    }

    // Add an edge case test for mixed mode handling
    #[tokio::test]
    async fn test_edge_case_mixed_mode() -> Result<()> {
        // Create a mock LLM that supports function calling but returns text with tool calls
        let mut llm = MockLlmProvider::new();
        llm.set_supports_function_calling(true);
        
        // Set up a response that has both function calls and tool call text
        llm.add_function_calling_response(
            vec![FunctionCall {
                id: Some("call_123".to_string()),
                name: "calculator".to_string(),
                arguments: r#"{"expression": "2+2"}"#.to_string(),
            }],
            Some("I'll calculate that. Using the tool 'weather' with parameters: {\"location\": \"New York\"}"),
        );
        
        // Create agent with both tools
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            enable_function_calling: Some(true),
            ..Default::default()
        };
        
        let agent = BasicAgent::new(config, Arc::new(llm));
        
        // Add both tools
        let mut tools = HashMap::new();
        tools.insert("calculator".to_string(), Box::new(TestTool::new("calculator", "A calculator tool")));
        tools.insert("weather".to_string(), Box::new(TestTool::new("weather", "A weather tool")));
        agent.set_tools(tools);
        
        // Execute test
        let result = agent.generate("Calculate 2+2 and check weather", &AgentGenerateOptions::default()).await?;
        
        // Should only see the function call since function calling is enabled
        assert_eq!(result.steps.len(), 1);
        assert_eq!(result.steps[0].tool_calls.len(), 1);
        assert_eq!(result.steps[0].tool_calls[0].name, "calculator");
        
        Ok(())
    }
}
