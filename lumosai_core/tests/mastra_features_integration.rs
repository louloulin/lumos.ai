//! Integration tests for Mastra-inspired features
//! 
//! This test suite validates the implementation of features inspired by Mastra

use lumosai_core::agent::{AgentBuilder, mastra_compat, Agent};
use lumosai_core::agent::trait_def::Agent as AgentTrait;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions};
use lumosai_core::tool::builtin::{
    create_all_builtin_tools, create_safe_builtin_tools, create_dev_builtin_tools,
    BuiltinToolsConfig
};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::test]
async fn test_agent_builder_pattern() {
    // Test the enhanced AgentBuilder pattern
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello from agent!".to_string()]));
    
    let agent = AgentBuilder::new()
        .name("test_agent")
        .instructions("You are a helpful assistant")
        .model(llm)
        .max_tool_calls(5)
        .tool_timeout(30)
        .enable_function_calling(true)
        .build()
        .expect("Failed to build agent");

    assert_eq!(agent.get_name(), "test_agent");
    assert_eq!(agent.get_instructions(), "You are a helpful assistant");
}

#[tokio::test]
async fn test_mastra_compatible_api() {
    // Test Mastra-compatible Agent creation API
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello from Mastra-style agent!".to_string()]));
    
    let agent = mastra_compat::Agent::create()
        .name("mastra_agent")
        .instructions("You are a Mastra-compatible assistant")
        .model(llm)
        .build()
        .expect("Failed to create Mastra-style agent");

    assert_eq!(agent.get_name(), "mastra_agent");
    assert_eq!(agent.get_instructions(), "You are a Mastra-compatible assistant");
}

#[tokio::test]
async fn test_agent_with_builtin_tools() {
    // Test agent creation with built-in tools
    let llm = Arc::new(MockLlmProvider::new(vec!["I'll use the tools to help you!".to_string()]));
    
    let agent = mastra_compat::Agent::with_tools()
        .name("tool_agent")
        .instructions("You are an agent with built-in tools")
        .model(llm)
        .with_math_tools()
        .with_data_tools()
        .with_system_tools()
        .build()
        .expect("Failed to create agent with tools");

    assert_eq!(agent.get_name(), "tool_agent");
    assert!(agent.get_tools().len() > 0);
    
    // Verify specific tools are available
    assert!(agent.get_tool("calculator").is_some());
    assert!(agent.get_tool("statistics").is_some());
    assert!(agent.get_tool("json_parser").is_some());
    assert!(agent.get_tool("datetime").is_some());
}

#[tokio::test]
async fn test_builtin_tools_creation() {
    // Test built-in tools creation functions
    let config = BuiltinToolsConfig::default();
    let all_tools = create_all_builtin_tools(&config);
    
    assert!(!all_tools.is_empty());
    assert!(all_tools.len() >= 15); // Should have at least 15 tools
    
    // Test tool names
    let tool_names: Vec<String> = all_tools.iter()
        .filter_map(|t| t.name().map(|n| n.to_string()))
        .collect();
    assert!(tool_names.contains(&"http_request".to_string()));
    assert!(tool_names.contains(&"file_reader".to_string()));
    assert!(tool_names.contains(&"calculator".to_string()));
    assert!(tool_names.contains(&"json_parser".to_string()));
    assert!(tool_names.contains(&"datetime".to_string()));
}

#[tokio::test]
async fn test_safe_vs_dev_tools() {
    // Test safe tools (production environment)
    let safe_tools = create_safe_builtin_tools(PathBuf::from("/tmp"));
    let safe_tool_names: Vec<String> = safe_tools.iter()
        .filter_map(|t| t.name().map(|n| n.to_string()))
        .collect();

    // Safe tools should not include file or web tools
    assert!(!safe_tool_names.contains(&"file_reader".to_string()));
    assert!(!safe_tool_names.contains(&"http_request".to_string()));

    // But should include data and math tools
    assert!(safe_tool_names.contains(&"calculator".to_string()));
    assert!(safe_tool_names.contains(&"json_parser".to_string()));

    // Test dev tools (development environment)
    let dev_tools = create_dev_builtin_tools();
    let dev_tool_names: Vec<String> = dev_tools.iter()
        .filter_map(|t| t.name().map(|n| n.to_string()))
        .collect();

    // Dev tools should include everything
    assert!(dev_tool_names.contains(&"file_reader".to_string()));
    assert!(dev_tool_names.contains(&"http_request".to_string()));
    assert!(dev_tool_names.contains(&"calculator".to_string()));
    assert!(dev_tool_names.contains(&"json_parser".to_string()));
}

#[tokio::test]
async fn test_mastra_utility_functions() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Quick response!".to_string()]));
    
    // Test quick agent creation
    let quick_agent = mastra_compat::utils::quick_agent(
        "quick_test",
        "Quick test agent",
        llm.clone(),
    ).expect("Failed to create quick agent");
    
    assert_eq!(quick_agent.get_name(), "quick_test");
    
    // Test agent with common tools
    let common_agent = mastra_compat::utils::agent_with_common_tools(
        "common_test",
        "Agent with common tools",
        llm.clone(),
    ).expect("Failed to create agent with common tools");
    
    assert_eq!(common_agent.get_name(), "common_test");
    assert!(common_agent.get_tools().len() > 0);
    
    // Test web agent
    let web_agent = mastra_compat::utils::web_agent(
        "web_test",
        "Web-enabled agent",
        llm.clone(),
    ).expect("Failed to create web agent");
    
    assert_eq!(web_agent.get_name(), "web_test");
    assert!(web_agent.get_tool("http_request").is_some());
    
    // Test file agent
    let file_agent = mastra_compat::utils::file_agent(
        "file_test",
        "File processing agent",
        llm,
    ).expect("Failed to create file agent");
    
    assert_eq!(file_agent.get_name(), "file_test");
    assert!(file_agent.get_tool("file_reader").is_some());
}

#[tokio::test]
async fn test_agent_config_creation() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Config-based agent!".to_string()]));
    
    let config = mastra_compat::AgentConfig::new("config_agent", "Config-based instructions")
        .with_model(llm)
        .with_memory(true);
    
    let agent = mastra_compat::Agent::from_config(config)
        .expect("Failed to create agent from config");
    
    assert_eq!(agent.get_name(), "config_agent");
    assert_eq!(agent.get_instructions(), "Config-based instructions");
}

#[tokio::test]
async fn test_tool_execution() {
    // Test that built-in tools can be executed
    let config = BuiltinToolsConfig::default();
    let tools = create_all_builtin_tools(&config);
    
    // Find and test the calculator tool
    let calculator = tools.into_iter()
        .find(|t| t.name() == Some("calculator"))
        .expect("Calculator tool not found");

    let params = serde_json::json!({
        "expression": "2 + 3"
    });

    let context = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    let result = calculator.execute(params, context, &options).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response["success"], true);
    assert_eq!(response["result"], 5.0);
}

#[tokio::test]
async fn test_comprehensive_agent_workflow() {
    // Test a comprehensive workflow that combines multiple features
    let llm = Arc::new(MockLlmProvider::new(vec![
        "I'll help you with calculations and data processing!".to_string(),
        "Let me use the calculator tool to compute 10 + 20".to_string(),
        "The result is 30. Now let me process some JSON data.".to_string(),
    ]));
    
    // Create an agent with multiple tool categories
    let agent = mastra_compat::Agent::with_tools()
        .name("comprehensive_agent")
        .instructions("You are a comprehensive assistant with multiple capabilities")
        .model(llm)
        .with_math_tools()
        .with_data_tools()
        .with_system_tools()
        .build()
        .expect("Failed to create comprehensive agent");
    
    // Verify the agent has the expected capabilities
    assert_eq!(agent.get_name(), "comprehensive_agent");
    assert!(agent.get_tools().len() >= 6); // Should have at least 6 tools
    
    // Verify specific tool categories are available
    assert!(agent.get_tool("calculator").is_some());
    assert!(agent.get_tool("statistics").is_some());
    assert!(agent.get_tool("json_parser").is_some());
    assert!(agent.get_tool("csv_parser").is_some());
    assert!(agent.get_tool("datetime").is_some());
    assert!(agent.get_tool("uuid_generator").is_some());
    
    // Test that the agent can generate responses
    let messages = vec![lumosai_core::llm::Message {
        role: lumosai_core::llm::Role::User,
        content: "Help me with some calculations and data processing".to_string(),
        metadata: None,
        name: None,
    }];
    
    let options = lumosai_core::agent::types::AgentGenerateOptions::default();
    let result = agent.generate(&messages, &options).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.response.is_empty());
}

#[tokio::test]
async fn test_performance_and_scalability() {
    // Test that we can create multiple agents efficiently
    let llm = Arc::new(MockLlmProvider::new(vec!["Performance test response".to_string()]));
    
    let start_time = std::time::Instant::now();
    
    // Create 10 agents with different configurations
    let mut agents = Vec::new();
    for i in 0..10 {
        let agent = mastra_compat::Agent::with_tools()
            .name(format!("perf_agent_{}", i))
            .instructions("Performance test agent")
            .model(llm.clone())
            .with_math_tools()
            .build()
            .expect("Failed to create performance test agent");
        
        agents.push(agent);
    }
    
    let creation_time = start_time.elapsed();
    
    // Should be able to create 10 agents quickly (under 1 second)
    assert!(creation_time.as_secs() < 1);
    assert_eq!(agents.len(), 10);
    
    // Verify all agents are properly configured
    for (i, agent) in agents.iter().enumerate() {
        assert_eq!(agent.get_name(), format!("perf_agent_{}", i));
        assert!(agent.get_tool("calculator").is_some());
    }
}
