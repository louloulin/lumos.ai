//! Plan4.md API Tests
//!
//! Comprehensive tests for the new API design specified in plan4.md

use super::*;
use crate::agent::trait_def::Agent as AgentTrait;
use crate::llm::MockLlmProvider;
use std::sync::Arc;
use tokio;

#[tokio::test]
async fn test_agent_factory_quick() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));

    let agent = AgentFactory::quick("test_agent", "You are a test assistant")
        .model(llm)
        .build()
        .expect("Failed to create agent");

    assert_eq!(agent.get_name(), "test_agent");
    assert_eq!(agent.get_instructions(), "You are a test assistant");

    // Test generate_simple method
    let response = agent.generate_simple("Hello").await.expect("Failed to generate response");
    assert_eq!(response, "Hello!");
}

#[tokio::test]
async fn test_agent_factory_builder() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Builder response".to_string()]));

    let agent = AgentFactory::builder()
        .name("builder_agent")
        .instructions("You are a builder test")
        .model(llm)
        .max_tool_calls(5)
        .tool_timeout(30)
        .build()
        .expect("Failed to create agent");

    assert_eq!(agent.get_name(), "builder_agent");
    assert_eq!(agent.get_instructions(), "You are a builder test");

    let response = agent.generate_simple("Test").await.expect("Failed to generate response");
    assert_eq!(response, "Builder response");
}



#[tokio::test]
async fn test_convenience_functions() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Convenience response".to_string()]));

    // Test quick function
    let quick_agent = quick("quick_test", "Quick test")
        .model(llm.clone())
        .build()
        .expect("Failed to create quick agent");

    assert_eq!(quick_agent.get_name(), "quick_test");

    let response = quick_agent.generate_simple("Test").await.expect("Failed to generate response");
    assert_eq!(response, "Convenience response");
}
