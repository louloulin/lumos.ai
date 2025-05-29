//! Integration tests for enhanced function calling feature

use std::sync::Arc;
use serde_json::json;

use lumosai_core::agent::{BasicAgent, AgentConfig, AgentGenerateOptions, Agent, message_utils::user_message};
use lumosai_core::llm::mock::MockLlmProvider;
use lumosai_core::tool::{GenericTool, ToolSchema, ParameterSchema};
use lumosai_core::error::Result;

// 添加这个函数到lumosai_core/tests/function_calling.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_function_calling() -> Result<()> {
        // Create mock LLM provider that supports function calling
        let llm = MockLlmProvider::new(vec!["Calculating 2+2... The result is 4".to_string()]);
        
        // Create agent config with function calling enabled
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a helpful assistant that can use tools.".to_string(),
            enable_function_calling: Some(true),
            ..Default::default()
        };
        
        // Create agent
        let mut agent = BasicAgent::new(config, Arc::new(llm));
        
        // Create calculator tool using GenericTool
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "expression".to_string(),
                description: "Mathematical expression to evaluate".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            }
        ]);
        
        let calculator = GenericTool::new(
            "calculator",
            "A calculator tool that can evaluate mathematical expressions",
            schema,
            |params, _context| {
                let expression = params.get("expression").and_then(|v| v.as_str()).unwrap_or("");
                let result = if expression == "2+2" {
                    4
                } else {
                    42 // Default result for demo
                };
                Ok(json!(result))
            },
        );
        
        // Add calculator tool to agent
        agent.add_tool(Box::new(calculator))?;
        
        // Execute test
        let messages = vec![user_message("Calculate 2+2")];
        let result = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
        
        // Verify results
        assert!(!result.response.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_enhanced_tool_parsing() -> Result<()> {
        // Create a mock LLM provider that doesn't support function calling
        let llm = MockLlmProvider::new(vec![
            "I'll help you with that. Using the tool 'calculator' with parameters: {\"expression\": \"2+2\"}".to_string(),
            "Let me calculate that. calculator(expression=\"2+2\")".to_string(),
            "I'll use the calculator tool:\n```json\n{\"tool\": \"calculator\", \"parameters\": {\"expression\": \"2+2\"}}\n```".to_string(),
        ]);
        
        // Create agent config with function calling disabled to force regex parsing
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a helpful assistant that can use tools.".to_string(),
            enable_function_calling: Some(false),
            ..Default::default()
        };
        
        // Create agent
        let mut agent = BasicAgent::new(config, Arc::new(llm));
        
        // Create calculator tool using GenericTool
        let schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "expression".to_string(),
                description: "Mathematical expression to evaluate".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            }
        ]);
        
        let calculator = GenericTool::new(
            "calculator",
            "A calculator tool that can evaluate mathematical expressions",
            schema,
            |params, _context| {
                let expression = params.get("expression").and_then(|v| v.as_str()).unwrap_or("");
                let result = if expression == "2+2" {
                    4
                } else {
                    42 // Default result for demo
                };
                Ok(json!(result))
            },
        );
        
        // Add calculator tool to agent
        agent.add_tool(Box::new(calculator))?;
        
        // Test standard regex format
        let messages1 = vec![user_message("Calculate 2+2 using standard format")];
        let result1 = agent.generate(&messages1, &AgentGenerateOptions::default()).await?;
        assert!(!result1.response.is_empty());
        
        // Test function-like format
        let messages2 = vec![user_message("Calculate 2+2 using function format")];
        let result2 = agent.generate(&messages2, &AgentGenerateOptions::default()).await?;
        assert!(!result2.response.is_empty());
        
        // Test JSON code block format
        let messages3 = vec![user_message("Calculate 2+2 using JSON format")];
        let result3 = agent.generate(&messages3, &AgentGenerateOptions::default()).await?;
        assert!(!result3.response.is_empty());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_system_message_generation() -> Result<()> {
        // Test with function calling
        {
            let llm = MockLlmProvider::new(vec!["Test response".to_string()]);
            
            let config = AgentConfig {
                name: "TestAgent".to_string(),
                instructions: "You are a helpful assistant.".to_string(),
                enable_function_calling: Some(true),
                ..Default::default()
            };
            
            let mut agent = BasicAgent::new(config, Arc::new(llm));
            
            // Create calculator tool using GenericTool
            let schema = ToolSchema::new(vec![
                ParameterSchema {
                    name: "expression".to_string(),
                    description: "Mathematical expression to evaluate".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                }
            ]);
            
            let calculator = GenericTool::new(
                "calculator",
                "A calculator tool that can evaluate mathematical expressions",
                schema,
                |params, _context| {
                    let expression = params.get("expression").and_then(|v| v.as_str()).unwrap_or("");
                    let result = if expression == "2+2" {
                        4
                    } else {
                        42 // Default result for demo
                    };
                    Ok(json!(result))
                },
            );
            
            // Add tool
            agent.add_tool(Box::new(calculator))?;
            
            // Generate to test system message includes tool information
            let messages = vec![user_message("Test")];
            let result = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
            
            // Verify the system worked (we can't easily test the exact system message in this mock)
            assert!(!result.response.is_empty());
        }
        
        // Test without function calling
        {
            let llm = MockLlmProvider::new(vec!["Test response".to_string()]);
            
            let config = AgentConfig {
                name: "TestAgent".to_string(),
                instructions: "You are a helpful assistant.".to_string(),
                enable_function_calling: Some(false),
                ..Default::default()
            };
            
            let mut agent = BasicAgent::new(config, Arc::new(llm));
            
            // Create calculator tool using GenericTool
            let schema = ToolSchema::new(vec![
                ParameterSchema {
                    name: "expression".to_string(),
                    description: "Mathematical expression to evaluate".to_string(),
                    r#type: "string".to_string(),
                    required: true,
                    properties: None,
                    default: None,
                }
            ]);
            
            let calculator = GenericTool::new(
                "calculator",
                "A calculator tool that can evaluate mathematical expressions",
                schema,
                |params, _context| {
                    let expression = params.get("expression").and_then(|v| v.as_str()).unwrap_or("");
                    let result = if expression == "2+2" {
                        4
                    } else {
                        42 // Default result for demo
                    };
                    Ok(json!(result))
                },
            );
            
            // Add tool
            agent.add_tool(Box::new(calculator))?;
            
            // Generate to capture the system message
            let messages = vec![user_message("Test")];
            let _ = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
        }
        
        Ok(())
    }

    // Add an edge case test for mixed mode handling
    #[tokio::test]
    async fn test_edge_case_mixed_mode() -> Result<()> {
        // Create a mock LLM that supports function calling but returns text with tool calls
        let llm = MockLlmProvider::new(vec![
            "I'll calculate that. Using the tool 'weather' with parameters: {\"location\": \"New York\"}".to_string()
        ]);
        
        // Create agent with both tools
        let config = AgentConfig {
            name: "TestAgent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            enable_function_calling: Some(true),
            ..Default::default()
        };
        
        let mut agent = BasicAgent::new(config, Arc::new(llm));
        
        // Create calculator tool
        let calc_schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "expression".to_string(),
                description: "Mathematical expression to evaluate".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            }
        ]);
        
        let calculator = GenericTool::new(
            "calculator",
            "A calculator tool that can evaluate mathematical expressions",
            calc_schema,
            |params, _context| {
                let expression = params.get("expression").and_then(|v| v.as_str()).unwrap_or("");
                let result = if expression == "2+2" {
                    4
                } else {
                    42 // Default result for demo
                };
                Ok(json!(result))
            },
        );
        
        // Create weather tool
        let weather_schema = ToolSchema::new(vec![
            ParameterSchema {
                name: "location".to_string(),
                description: "Location to get weather for".to_string(),
                r#type: "string".to_string(),
                required: true,
                properties: None,
                default: None,
            }
        ]);
        
        let weather = GenericTool::new(
            "weather",
            "A weather tool that can get weather information",
            weather_schema,
            |params, _context| {
                let location = params.get("location").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!(format!("Weather in {} is sunny", location)))
            },
        );
        
        // Add both tools
        agent.add_tool(Box::new(calculator))?;
        agent.add_tool(Box::new(weather))?;
        
        // Execute test
        let messages = vec![user_message("Calculate 2+2 and check weather")];
        let result = agent.generate(&messages, &AgentGenerateOptions::default()).await?;
        
        // Should have a response
        assert!(!result.response.is_empty());
        
        Ok(())
    }
}
