//! Plan4.md API Tests
//!
//! Comprehensive tests for the new API design specified in plan4.md

use super::*;
use crate::agent::trait_def::Agent as AgentTrait;
use crate::llm::test_helpers::create_test_zhipu_provider_arc;
use std::sync::Arc;
use std::time::Duration;
use tokio;

#[tokio::test]
async fn test_agent_factory_quick() {
    let llm = create_test_zhipu_provider_arc();

    let agent = AgentFactory::quick("test_agent", "You are a test assistant")
        .model(llm)
        .build()
        .expect("Failed to create agent");

    assert_eq!(agent.get_name(), "test_agent");
    assert_eq!(agent.get_instructions(), "You are a test assistant");

    // Test generate_simple method
    let response = agent
        .generate_simple("Hello")
        .await
        .expect("Failed to generate response");

    // Real LLM returns variable responses, just check it's not empty
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.len() > 5, "Response should be meaningful");
}

#[tokio::test]
async fn test_agent_factory_builder() {
    let llm = create_test_zhipu_provider_arc();

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

    let response = agent
        .generate_simple("Test")
        .await
        .expect("Failed to generate response");

    // Real LLM returns variable responses, just check it's not empty
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.len() > 5, "Response should be meaningful");
}

#[tokio::test]
async fn test_convenience_functions() {
    let llm = create_test_zhipu_provider_arc();

    // Test quick function
    let quick_agent = quick("quick_test", "Quick test")
        .model(llm.clone())
        .build()
        .expect("Failed to create quick agent");

    assert_eq!(quick_agent.get_name(), "quick_test");

    tokio::time::sleep(Duration::from_millis(1000)).await;
    let response = quick_agent
        .generate_simple("Test")
        .await
        .expect("Failed to generate response");

    // Real LLM returns variable responses, just check it's not empty
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.len() > 5, "Response should be meaningful");
}
