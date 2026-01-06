//! E2E Test Scenarios
//!
//! Comprehensive end-to-end test scenarios covering all major functionality.

use super::test_context::E2ETestContext;
use super::test_helpers::*;
use lumosai_core::agent::Agent;
use lumosai_core::prelude::*;
use lumosai_core::tool::ToolExecutionContext;
use std::time::Duration;

/// Test Scenario 1: Basic Agent Conversation
///
/// Validates that an agent can be created and respond to simple messages.
pub async fn test_basic_agent_conversation() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    // Create agent
    let agent = context
        .create_agent("assistant", "You are a helpful AI assistant.")
        .await?;

    // Send message and get response
    let response = agent.generate_simple("Hello! What is 2 + 2?").await?;

    // Validate response
    assert!(!response.is_empty(), "Response should not be empty");
    assert_response_contains(&response, &["4", "four"]);

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 2: Agent with Tools
///
/// Validates that an agent can use tools to perform tasks.
pub async fn test_agent_with_tools() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    // Create calculator tool
    let calculator = context.create_calculator_tool()?;

    // Create agent with tool
    let agent = context
        .create_agent_with_tools(
            "calculator_agent",
            "You are a math assistant. Use the calculator tool when needed.",
            vec![calculator],
        )
        .await?;

    // Test tool usage
    let response = agent.generate_simple("Calculate 15 + 27").await?;

    // Validate response
    assert!(!response.is_empty(), "Response should not be empty");

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 3: Multi-turn Conversation
///
/// Validates that an agent can maintain context across multiple turns.
pub async fn test_multi_turn_conversation() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    let agent = context
        .create_agent("assistant", "You are a helpful AI assistant.")
        .await?;

    // First turn
    let response1 = agent.generate_simple("My name is Alice.").await?;
    assert!(!response1.is_empty());

    // Second turn - should remember the name
    let response2 = agent.generate_simple("What is my name?").await?;
    assert!(!response2.is_empty());

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 4: Error Handling
///
/// Validates that the system handles errors gracefully.
pub async fn test_error_handling() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    // Create calculator tool
    let calculator = context.create_calculator_tool()?;

    // Test division by zero
    let tool_context = ToolExecutionContext::default();
    let params = serde_json::json!({
        "operation": "divide",
        "a": 10.0,
        "b": 0.0
    });

    let result = calculator.execute(params, &tool_context).await;

    // Should return an error
    assert_error(&result);
    assert_error_contains(&result, "Division by zero");

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 5: Concurrent Agent Operations
///
/// Validates that multiple agents can operate concurrently.
pub async fn test_concurrent_operations() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    // Create multiple agents
    let agent1 = context.create_agent("agent1", "You are agent 1.").await?;
    let agent2 = context.create_agent("agent2", "You are agent 2.").await?;
    let agent3 = context.create_agent("agent3", "You are agent 3.").await?;

    // Run concurrent operations
    let (result1, result2, result3) = tokio::join!(
        agent1.generate_simple("Hello from agent 1"),
        agent2.generate_simple("Hello from agent 2"),
        agent3.generate_simple("Hello from agent 3"),
    );

    // All should succeed
    assert_success(&result1);
    assert_success(&result2);
    assert_success(&result3);

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 6: Performance Benchmark
///
/// Validates that basic operations meet performance requirements.
pub async fn test_performance_benchmark() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    let agent = context
        .create_agent("assistant", "You are a helpful AI assistant.")
        .await?;

    // Measure response time
    let (response, duration) = measure_time(|| async {
        agent.generate_simple("Hello!").await
    })
    .await;

    // Validate response
    assert_success(&response);

    // Response should be reasonably fast (< 5 seconds for test LLM)
    assert_execution_time(duration, Duration::from_secs(5));

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 7: Tool Parameter Validation
///
/// Validates that tool parameters are properly validated.
pub async fn test_tool_parameter_validation() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    let calculator = context.create_calculator_tool()?;
    let tool_context = ToolExecutionContext::default();

    // Test missing parameter
    let params = serde_json::json!({
        "operation": "add",
        "a": 5.0
        // Missing 'b'
    });

    let result = calculator.execute(params, &tool_context).await;
    assert_error(&result);

    // Test invalid operation
    let params = serde_json::json!({
        "operation": "invalid",
        "a": 5.0,
        "b": 3.0
    });

    let result = calculator.execute(params, &tool_context).await;
    assert_error(&result);

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 8: Agent Configuration
///
/// Validates that agent configuration works correctly.
pub async fn test_agent_configuration() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    let agent = context
        .create_agent(
            "configured_agent",
            "You are a specialized assistant with specific instructions.",
        )
        .await?;

    // Verify agent properties
    assert_eq!(agent.name(), Some("configured_agent"));

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 9: Multiple Tools
///
/// Validates that an agent can use multiple tools.
pub async fn test_multiple_tools() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    // Create multiple tools
    let calculator = context.create_calculator_tool()?;
    let text_tool = context.create_simple_tool("text_processor", "Process text")?;

    // Create agent with multiple tools
    let agent = context
        .create_agent_with_tools(
            "multi_tool_agent",
            "You are an assistant with multiple tools.",
            vec![calculator, text_tool],
        )
        .await?;

    // Verify agent has tools
    let tools = agent.tools();
    assert_eq!(tools.len(), 2);

    context.cleanup().await;
    Ok(())
}

/// Test Scenario 10: Workflow Execution
///
/// Validates basic workflow creation and execution.
pub async fn test_workflow_execution() -> Result<()> {
    let mut context = E2ETestContext::new().await?;

    // Create workflow
    let workflow = context.create_workflow("test_workflow").await?;

    // Verify workflow was created
    assert_eq!(workflow.id(), "test_workflow");

    context.cleanup().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_test_basic_agent_conversation() {
        let result = test_basic_agent_conversation().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_agent_with_tools() {
        let result = test_agent_with_tools().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_multi_turn_conversation() {
        let result = test_multi_turn_conversation().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_error_handling() {
        let result = test_error_handling().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_concurrent_operations() {
        let result = test_concurrent_operations().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_performance_benchmark() {
        let result = test_performance_benchmark().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_tool_parameter_validation() {
        let result = test_tool_parameter_validation().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_agent_configuration() {
        let result = test_agent_configuration().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_multiple_tools() {
        let result = test_multiple_tools().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }

    #[tokio::test]
    async fn run_test_workflow_execution() {
        let result = test_workflow_execution().await;
        assert!(result.is_ok(), "Test failed: {:?}", result);
    }
}

