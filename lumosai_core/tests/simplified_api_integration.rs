//! Integration tests for the simplified API (plan4.md implementation)
//! 
//! These tests verify that the new simplified API works correctly and provides
//! the expected developer experience improvements.

use lumosai_core::agent::{Agent, web_agent, file_agent, data_agent};
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::Agent as AgentTrait;
use std::sync::Arc;

#[tokio::test]
async fn test_quick_agent_creation() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    let agent = Agent::quick("test_agent", "You are a test assistant")
        .model(llm)
        .build()
        .expect("Failed to create quick agent");

    assert_eq!(agent.get_name(), "test_agent");
    assert_eq!(agent.get_instructions(), "You are a test assistant");
}

#[tokio::test]
async fn test_builder_pattern_agent() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    let agent = Agent::builder()
        .name("builder_agent")
        .instructions("You are a builder test assistant")
        .model(llm)
        .max_tool_calls(5)
        .build()
        .expect("Failed to create builder agent");

    assert_eq!(agent.get_name(), "builder_agent");
    assert_eq!(agent.get_instructions(), "You are a builder test assistant");
}

#[tokio::test]
async fn test_web_agent_with_tools() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    let agent = web_agent("web_test", "You are a web assistant")
        .model(llm)
        .build()
        .expect("Failed to create web agent");

    assert_eq!(agent.get_name(), "web_test");
    assert!(agent.get_instructions().contains("web"));
    
    // Verify web tools are available
    let tools = agent.get_tools();
    assert!(tools.len() > 0, "Web agent should have tools");
    
    // Check for specific web tools
    assert!(tools.contains_key("http_request"), "Should have http_request tool");
    assert!(tools.contains_key("web_scraper"), "Should have web_scraper tool");
    assert!(tools.contains_key("json_api"), "Should have json_api tool");
    assert!(tools.contains_key("url_validator"), "Should have url_validator tool");
}

#[tokio::test]
async fn test_file_agent_with_tools() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    let agent = file_agent("file_test", "You are a file assistant")
        .model(llm)
        .build()
        .expect("Failed to create file agent");

    assert_eq!(agent.get_name(), "file_test");
    assert!(agent.get_instructions().contains("file"));
    
    // Verify file tools are available
    let tools = agent.get_tools();
    assert!(tools.len() > 0, "File agent should have tools");
    
    // Check for specific file tools
    assert!(tools.contains_key("file_reader"), "Should have file_reader tool");
    assert!(tools.contains_key("file_writer"), "Should have file_writer tool");
    assert!(tools.contains_key("directory_lister"), "Should have directory_lister tool");
    assert!(tools.contains_key("file_info"), "Should have file_info tool");
}

#[tokio::test]
async fn test_data_agent_with_tools() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    let agent = data_agent("data_test", "You are a data assistant")
        .model(llm)
        .build()
        .expect("Failed to create data agent");

    assert_eq!(agent.get_name(), "data_test");
    assert!(agent.get_instructions().contains("data"));
    
    // Verify data tools are available
    let tools = agent.get_tools();
    assert!(tools.len() > 0, "Data agent should have tools");
    
    // Check for specific data tools
    assert!(tools.contains_key("json_parser"), "Should have json_parser tool");
    assert!(tools.contains_key("csv_parser"), "Should have csv_parser tool");
    assert!(tools.contains_key("data_transformer"), "Should have data_transformer tool");
}

#[tokio::test]
async fn test_smart_defaults_applied() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    let agent = Agent::quick("smart_test", "You are a smart assistant")
        .model(llm)
        .build()
        .expect("Failed to create smart agent");

    // Verify smart defaults are applied
    // Note: These would need to be exposed through the Agent trait or builder
    // For now, we just verify the agent was created successfully
    assert_eq!(agent.get_name(), "smart_test");
    assert_eq!(agent.get_instructions(), "You are a smart assistant");
}

#[tokio::test]
async fn test_agent_interaction() {
    let llm = Arc::new(MockLlmProvider::new(vec![
        "Hello! I'm your AI assistant.".to_string(),
    ]));
    
    let agent = Agent::quick("interaction_test", "You are a helpful assistant")
        .model(llm)
        .build()
        .expect("Failed to create agent");

    // Test basic interaction
    let response = agent.generate("Hello, can you introduce yourself?", None).await;
    assert!(response.is_ok(), "Agent should respond successfully");
    
    let response_text = response.unwrap();
    assert!(!response_text.is_empty(), "Response should not be empty");
}

#[tokio::test]
async fn test_backward_compatibility() {
    // Test that old-style agent creation still works
    use lumosai_core::agent::{AgentBuilder, create_basic_agent};
    
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    // Old-style creation should still work
    let old_agent = create_basic_agent(
        "old_style".to_string(),
        "You are an old-style agent".to_string(),
        llm.clone()
    );
    
    assert_eq!(old_agent.get_name(), "old_style");
    assert_eq!(old_agent.get_instructions(), "You are an old-style agent");
    
    // Builder pattern should also work
    let builder_agent = AgentBuilder::new()
        .name("builder_test")
        .instructions("You are a builder agent")
        .model(llm)
        .build()
        .expect("Failed to create builder agent");
    
    assert_eq!(builder_agent.get_name(), "builder_test");
    assert_eq!(builder_agent.get_instructions(), "You are a builder agent");
}

#[tokio::test]
async fn test_api_simplicity_comparison() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    // New simplified API - should be very concise
    let simple_agent = Agent::quick("simple", "You are simple")
        .model(llm.clone())
        .build()
        .expect("Failed to create simple agent");
    
    // Verify it works the same as more verbose creation
    let verbose_agent = Agent::builder()
        .name("simple")
        .instructions("You are simple")
        .model(llm)
        .enable_smart_defaults()
        .build()
        .expect("Failed to create verbose agent");
    
    assert_eq!(simple_agent.get_name(), verbose_agent.get_name());
    assert_eq!(simple_agent.get_instructions(), verbose_agent.get_instructions());
}

#[tokio::test]
async fn test_performance_no_regression() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    // Test that simplified API doesn't add performance overhead
    let start = std::time::Instant::now();
    
    for i in 0..100 {
        let _agent = Agent::quick(&format!("perf_test_{}", i), "You are fast")
            .model(llm.clone())
            .build()
            .expect("Failed to create performance test agent");
    }
    
    let duration = start.elapsed();
    
    // Should be able to create 100 agents quickly (under 1 second)
    assert!(duration.as_secs() < 1, "Agent creation should be fast: {:?}", duration);
}

#[tokio::test]
async fn test_error_handling_improvements() {
    // Test that error messages are helpful
    let result = Agent::quick("test", "instructions")
        .build(); // Missing model - should give helpful error
    
    assert!(result.is_err(), "Should fail without model");
    
    let error = result.unwrap_err();
    let error_msg = format!("{}", error);
    
    // Error message should be helpful (this is a basic check)
    assert!(!error_msg.is_empty(), "Error message should not be empty");
}
