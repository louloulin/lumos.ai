// Unit tests for Agent system
use crate::test_config::*;
use lumosai_core::prelude::*;
use std::time::Duration;

#[tokio::test]
async fn test_agent_creation() {
    init_test_env();
    
    let agent = Agent::builder()
        .name("test-agent")
        .model("gpt-4")
        .system_prompt("You are a helpful assistant")
        .build()
        .await;
    
    assert!(agent.is_ok(), "Agent creation should succeed");
    
    let agent = agent.unwrap();
    assert_eq!(agent.get_name(), "test-agent");
}

#[tokio::test]
async fn test_agent_creation_with_invalid_params() {
    init_test_env();
    
    // Test with empty name
    let result = Agent::builder()
        .name("")
        .model("gpt-4")
        .build()
        .await;
    
    assert!(result.is_err(), "Agent creation with empty name should fail");
}

#[tokio::test]
async fn test_agent_simple_generation() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("test-agent").await.unwrap();
    
    let response = agent.generate_simple("Hello, how are you?").await;
    assert!(response.is_ok(), "Agent generation should succeed");
    
    let response = response.unwrap();
    TestAssertions::assert_valid_agent_response(&response);
}

#[tokio::test]
async fn test_agent_generation_with_empty_input() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("test-agent").await.unwrap();
    
    let response = agent.generate_simple("").await;
    // Should handle empty input gracefully
    assert!(response.is_ok(), "Agent should handle empty input gracefully");
}

#[tokio::test]
async fn test_agent_generation_timeout() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("test-agent").await.unwrap();
    
    // Test with timeout
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        agent.generate_simple("Test message")
    ).await;
    
    assert!(result.is_ok(), "Agent generation should complete within timeout");
}

#[tokio::test]
async fn test_agent_multiple_generations() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("test-agent").await.unwrap();
    
    // Test multiple generations
    for i in 0..5 {
        let message = format!("Test message {}", i);
        let response = agent.generate_simple(&message).await;
        
        assert!(response.is_ok(), "Generation {} should succeed", i);
        TestAssertions::assert_valid_agent_response(&response.unwrap());
    }
}

#[tokio::test]
async fn test_agent_concurrent_generations() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("test-agent").await.unwrap();
    
    // Test concurrent generations
    let mut handles = Vec::new();
    
    for i in 0..3 {
        let agent_clone = agent.clone();
        let message = format!("Concurrent test message {}", i);
        
        let handle = tokio::spawn(async move {
            agent_clone.generate_simple(&message).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent generation {} should succeed", i);
        
        let response = result.unwrap();
        assert!(response.is_ok(), "Response {} should be valid", i);
    }
}

#[tokio::test]
async fn test_agent_builder_pattern() {
    init_test_env();
    
    // Test builder pattern with various configurations
    let agent = Agent::builder()
        .name("builder-test-agent")
        .model("gpt-4")
        .system_prompt("You are a specialized assistant")
        .build()
        .await;
    
    assert!(agent.is_ok(), "Builder pattern should work");
    
    let agent = agent.unwrap();
    assert_eq!(agent.get_name(), "builder-test-agent");
}

#[tokio::test]
async fn test_agent_error_handling() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("error-test-agent").await.unwrap();
    
    // Test with very long input (should handle gracefully)
    let long_input = "x".repeat(10000);
    let response = agent.generate_simple(&long_input).await;
    
    // Should either succeed or fail gracefully
    match response {
        Ok(resp) => TestAssertions::assert_valid_agent_response(&resp),
        Err(_) => {
            // Error is acceptable for very long input
            println!("Long input handled with error (acceptable)");
        }
    }
}

#[tokio::test]
async fn test_agent_memory_integration() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("memory-test-agent").await.unwrap();
    
    // Test that agent can maintain context (if memory is implemented)
    let first_response = agent.generate_simple("My name is Alice").await;
    assert!(first_response.is_ok(), "First generation should succeed");
    
    let second_response = agent.generate_simple("What is my name?").await;
    assert!(second_response.is_ok(), "Second generation should succeed");
    
    // Note: This test assumes memory functionality is implemented
    // If not implemented yet, this test documents the expected behavior
}

#[tokio::test]
async fn test_agent_performance_baseline() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("perf-test-agent").await.unwrap();
    
    let (response, duration) = PerformanceTestUtils::measure_time(|| async {
        agent.generate_simple("Quick test message").await
    }).await;
    
    assert!(response.is_ok(), "Performance test generation should succeed");
    
    // Assert reasonable response time (adjust based on your requirements)
    PerformanceTestUtils::assert_execution_time_within(
        duration,
        Duration::from_secs(10) // 10 seconds max for test environment
    );
    
    println!("Agent generation took: {:?}", duration);
}

#[tokio::test]
async fn test_agent_stress_test() {
    init_test_env();
    
    let agent = TestUtils::create_test_agent("stress-test-agent").await.unwrap();
    
    // Stress test with multiple rapid generations
    let iterations = 10;
    let mut success_count = 0;
    
    for i in 0..iterations {
        let message = format!("Stress test message {}", i);
        match agent.generate_simple(&message).await {
            Ok(_) => success_count += 1,
            Err(e) => println!("Stress test iteration {} failed: {:?}", i, e),
        }
    }
    
    // Allow some failures under stress, but expect majority to succeed
    let success_rate = success_count as f64 / iterations as f64;
    assert!(
        success_rate >= 0.7,
        "Expected at least 70% success rate under stress, got {:.1}%",
        success_rate * 100.0
    );
    
    println!("Stress test success rate: {:.1}%", success_rate * 100.0);
}
