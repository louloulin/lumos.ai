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

/// Helper function to retry API calls with exponential backoff
async fn retry_with_backoff<F, Fut, T>(
    mut f: F,
    max_retries: u32,
    initial_delay_ms: u64,
) -> crate::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = crate::Result<T>>,
{
    let mut delay = initial_delay_ms;
    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                let error_msg = format!("{:?}", e);
                if error_msg.contains("429") || error_msg.contains("Too Many Requests") || error_msg.contains("1302") {
                    if attempt < max_retries - 1 {
                        eprintln!("⚠️  Rate limit hit (attempt {}/{}), retrying after {}ms...", attempt + 1, max_retries, delay);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        delay *= 2; // Exponential backoff
                        continue;
                    }
                }
                return Err(e);
            }
        }
    }
    unreachable!()
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

    let result = retry_with_backoff(
        || async { quick_agent.generate_simple("Test").await },
        5,
        2000,
    ).await;

    assert!(result.is_ok(), "Failed with error: {:?}", result.err());
    let response = result.unwrap();

    // Real LLM returns variable responses, just check it's not empty
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.len() > 5, "Response should be meaningful");
}
