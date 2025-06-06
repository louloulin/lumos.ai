//! Integration tests for the simplified prelude API (plan6.md Phase 1)
//! 
//! These tests verify that the Rig-inspired simplified API works correctly
//! and provides the developer experience improvements outlined in plan6.md.

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::agent::trait_def::Agent as AgentTrait;
use std::sync::Arc;

#[tokio::test]
async fn test_quick_agent_creation() {
    // Test the most basic API from plan6.md
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello from quick agent!".to_string()]));
    
    let agent = quick_agent("assistant", "You are a helpful assistant")
        .model(llm)
        .build()
        .expect("Failed to create quick agent");

    assert_eq!(agent.get_name(), "assistant");
    assert_eq!(agent.get_instructions(), "You are a helpful assistant");
    
    // Test that the agent can generate responses
    let response = agent.generate_simple("Hello").await.expect("Failed to generate response");
    assert_eq!(response, "Hello from quick agent!");
}

#[tokio::test]
async fn test_agent_quick_static_method() {
    // Test Agent::quick() static method
    let llm = Arc::new(MockLlmProvider::new(vec!["Response from static method".to_string()]));
    
    let agent = Agent::quick("test_agent", "Test instructions")
        .model(llm)
        .build()
        .expect("Failed to create agent with static method");

    assert_eq!(agent.get_name(), "test_agent");
    assert_eq!(agent.get_instructions(), "Test instructions");
}

#[tokio::test]
async fn test_agent_builder_pattern() {
    // Test the full builder pattern
    let llm = Arc::new(MockLlmProvider::new(vec!["Builder pattern response".to_string()]));
    
    let agent = Agent::builder()
        .name("builder_agent")
        .instructions("Built with builder pattern")
        .model(llm)
        .max_tool_calls(5)
        .build()
        .expect("Failed to create agent with builder pattern");

    assert_eq!(agent.get_name(), "builder_agent");
    assert_eq!(agent.get_instructions(), "Built with builder pattern");
}

#[tokio::test]
async fn test_web_agent_quick() {
    // Test web agent convenience function
    let llm = Arc::new(MockLlmProvider::new(vec!["Web agent response".to_string()]));
    
    let agent = web_agent_quick("web_helper", "You can browse the web")
        .model(llm)
        .build()
        .expect("Failed to create web agent");

    assert_eq!(agent.get_name(), "web_helper");
    assert_eq!(agent.get_instructions(), "You can browse the web");

    // Should have web tools
    let tools = agent.get_tools();
    assert!(tools.len() > 0, "Web agent should have tools");
    
    // Check that web tools are present
    let tool_names: Vec<String> = tools.iter().map(|(name, _)| name.clone()).collect();
    assert!(tool_names.contains(&"http_request".to_string()));
    assert!(tool_names.contains(&"web_scraper".to_string()));
}

#[tokio::test]
async fn test_file_agent_quick() {
    // Test file agent convenience function
    let llm = Arc::new(MockLlmProvider::new(vec!["File agent response".to_string()]));
    
    let agent = file_agent_quick("file_helper", "You can manage files")
        .model(llm)
        .build()
        .expect("Failed to create file agent");

    assert_eq!(agent.get_name(), "file_helper");
    assert_eq!(agent.get_instructions(), "You can manage files");

    // Should have file tools
    let tools = agent.get_tools();
    assert!(tools.len() > 0, "File agent should have tools");
    
    // Check that file tools are present
    let tool_names: Vec<String> = tools.iter().map(|(name, _)| name.clone()).collect();
    assert!(tool_names.contains(&"file_reader".to_string()));
    assert!(tool_names.contains(&"file_writer".to_string()));
}

#[tokio::test]
async fn test_data_agent_quick() {
    // Test data agent convenience function
    let llm = Arc::new(MockLlmProvider::new(vec!["Data agent response".to_string()]));
    
    let agent = data_agent_quick("data_helper", "You can process data")
        .model(llm)
        .build()
        .expect("Failed to create data agent");

    assert_eq!(agent.get_name(), "data_helper");
    assert_eq!(agent.get_instructions(), "You can process data");

    // Should have data and math tools
    let tools = agent.get_tools();
    assert!(tools.len() > 0, "Data agent should have tools");
    
    // Check that data tools are present
    let tool_names: Vec<String> = tools.iter().map(|(name, _)| name.clone()).collect();
    assert!(tool_names.contains(&"json_parser".to_string()));
    assert!(tool_names.contains(&"calculator".to_string()));
}

#[tokio::test]
async fn test_tool_convenience_functions() {
    // Test that tool convenience functions work
    let web_tool = web_search();
    assert_eq!(web_tool.name(), Some("http_request"));

    let file_tool = file_reader();
    assert_eq!(file_tool.name(), Some("file_reader"));

    let data_tool = json_parser();
    assert_eq!(data_tool.name(), Some("json_parser"));

    let math_tool = calculator();
    assert_eq!(math_tool.name(), Some("calculator"));

    let system_tool = uuid_generator();
    assert_eq!(system_tool.name(), Some("uuid_generator"));
}

#[tokio::test]
async fn test_agent_with_custom_tools() {
    // Test adding custom tools to an agent
    let llm = Arc::new(MockLlmProvider::new(vec!["Custom tools response".to_string()]));
    
    let agent = Agent::quick("custom_agent", "Agent with custom tools")
        .model(llm)
        .tools(vec![
            web_search(),
            calculator(),
            json_parser(),
        ])
        .build()
        .expect("Failed to create agent with custom tools");

    assert_eq!(agent.get_name(), "custom_agent");

    // Should have exactly 3 tools
    let tools = agent.get_tools();
    assert_eq!(tools.len(), 3, "Agent should have exactly 3 tools");
    
    let tool_names: Vec<String> = tools.iter().map(|(name, _)| name.clone()).collect();
    assert!(tool_names.contains(&"http_request".to_string()));
    assert!(tool_names.contains(&"calculator".to_string()));
    assert!(tool_names.contains(&"json_parser".to_string()));
}

#[tokio::test]
async fn test_rig_style_api_comparison() {
    // Test that our API is as simple as Rig's
    let llm = Arc::new(MockLlmProvider::new(vec!["Rig-style response".to_string()]));
    
    // This should be as simple as Rig's API:
    // let agent = Agent::quick("assistant", "You are helpful").model("gpt-4").build()?;
    let agent = Agent::quick("assistant", "You are helpful")
        .model(llm)
        .build()
        .expect("Failed to create Rig-style agent");
    
    let response = agent.generate_simple("Hello").await.expect("Failed to generate");
    assert_eq!(response, "Rig-style response");
    
    // Verify the agent was created correctly
    assert_eq!(agent.get_name(), "assistant");
    assert_eq!(agent.get_instructions(), "You are helpful");
}

#[tokio::test]
async fn test_error_handling() {
    // Test that errors are handled gracefully
    let llm = Arc::new(MockLlmProvider::new(vec![])); // Empty responses to trigger error

    let agent = Agent::quick("error_test", "Test error handling")
        .model(llm)
        .build()
        .expect("Failed to create agent for error test");

    // This should handle the error gracefully
    let result = agent.generate_simple("This should fail").await;
    // MockLlmProvider returns "Default mock response" when no responses are available
    match result {
        Ok(response) => assert_eq!(response, "Default mock response", "Should return default mock response for empty mock responses"),
        Err(_) => (), // Error is also acceptable
    }
}
