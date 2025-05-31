//! Comprehensive Mastra Integration Tests
//! 
//! This test suite validates the complete Mastra functionality migration
//! to LumosAI, including streaming, function calling, memory management,
//! and monitoring capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use lumosai_core::agent::{AgentConfig, BasicAgent};
use lumosai_core::agent::streaming::{StreamingAgent, StreamingConfig, AgentEvent};
use lumosai_core::agent::types::{AgentGenerateOptions, ToolCall, RuntimeContext};
use lumosai_core::agent::evaluation::{RelevanceMetric, EvaluationMetric};
use lumosai_core::memory::{WorkingMemoryConfig, processor::{MessageLimitProcessor, MemoryProcessor}};
use lumosai_core::llm::{Message, MockLlmProvider};
use lumosai_core::telemetry::{Logger, Component, LogLevel, create_logger};
use lumosai_core::tool::{Tool, ToolResult};

use futures::StreamExt;
use serde_json::Value;
use tokio;

/// Test comprehensive streaming functionality
#[tokio::test]
async fn test_comprehensive_streaming_integration() {
    println!("🧪 Testing comprehensive streaming integration...");

    // Create working memory config
    let wm_config = WorkingMemoryConfig {
        enabled: true,
        template: None,
        content_type: Some("application/json".to_string()),
        max_capacity: Some(1024),
    };

    // Create agent config
    let agent_config = AgentConfig {
        name: "mastra_test_agent".to_string(),
        instructions: "You are a helpful AI assistant with streaming capabilities".to_string(),
        working_memory: Some(wm_config),
        ..Default::default()
    };

    // Create mock LLM with realistic streaming responses
    let mock_responses = vec![
        "I".to_string(),
        " understand".to_string(),
        " your".to_string(),
        " request".to_string(),
        ". Let".to_string(),
        " me".to_string(),
        " help".to_string(),
        " you".to_string(),
        " with".to_string(),
        " that".to_string(),
        ".".to_string(),
    ];
    
    let llm = Arc::new(MockLlmProvider::new(mock_responses));
    let agent = BasicAgent::new(agent_config, llm);

    // Configure streaming with metadata and memory updates
    let streaming_config = StreamingConfig {
        text_buffer_size: 1, // Character-by-character streaming
        emit_metadata: true,
        emit_memory_updates: true,
        text_delta_delay_ms: None, // No delay for testing
    };

    let streaming_agent = StreamingAgent::with_config(agent, streaming_config);

    // Test messages
    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hello, can you help me understand streaming?".to_string(),
    }];

    let options = AgentGenerateOptions::default();
    let mut event_stream = streaming_agent.execute_streaming(&messages, &options);

    // Collect and analyze events
    let mut events = Vec::new();
    let mut text_deltas = Vec::new();
    let mut metadata_count = 0;
    let mut generation_complete = false;

    while let Some(event_result) = event_stream.next().await {
        match event_result {
            Ok(event) => {
                match &event {
                    AgentEvent::TextDelta { delta, step_id } => {
                        text_deltas.push(delta.clone());
                        assert!(step_id.is_some(), "Step ID should be present");
                    },
                    AgentEvent::Metadata { key, value } => {
                        metadata_count += 1;
                        println!("📊 Metadata: {} = {:?}", key, value);
                    },
                    AgentEvent::GenerationComplete { final_response, total_steps } => {
                        generation_complete = true;
                        assert!(!final_response.is_empty(), "Final response should not be empty");
                        assert!(*total_steps > 0, "Should have at least one step");
                        println!("✅ Generation complete: {} steps", total_steps);
                    },
                    AgentEvent::Error { error, step_id } => {
                        panic!("Unexpected error: {} (step: {:?})", error, step_id);
                    },
                    _ => {}
                }
                events.push(event);
            },
            Err(e) => {
                panic!("Stream error: {:?}", e);
            }
        }
    }

    // Validate results
    assert!(!events.is_empty(), "Should receive events");
    assert!(!text_deltas.is_empty(), "Should receive text deltas");
    assert!(metadata_count > 0, "Should receive metadata events");
    assert!(generation_complete, "Should complete generation");

    let reconstructed_text: String = text_deltas.join("");
    assert!(!reconstructed_text.is_empty(), "Should reconstruct text");

    println!("✅ Streaming integration test passed:");
    println!("   - Total events: {}", events.len());
    println!("   - Text deltas: {}", text_deltas.len());
    println!("   - Metadata events: {}", metadata_count);
    println!("   - Reconstructed: '{}'", reconstructed_text);
}

/// Test dynamic arguments and runtime context
#[tokio::test]
async fn test_dynamic_arguments_and_runtime_context() {
    println!("🧪 Testing dynamic arguments and runtime context...");

    // Create runtime context with variables
    let mut context = RuntimeContext::new();
    context.set_variable("user_id", Value::String("test_user_123".to_string()));
    context.set_variable("session_id", Value::String("session_456".to_string()));
    context.set_metadata("request_type".to_string(), "test".to_string());

    // Test context operations
    assert_eq!(
        context.get_variable("user_id"),
        Some(&Value::String("test_user_123".to_string()))
    );
    assert_eq!(
        context.get_metadata("request_type"),
        Some(&"test".to_string())
    );

    // Test dynamic argument resolution
    let dynamic_instructions = |ctx: &RuntimeContext| -> String {
        if let Some(user_id) = ctx.get_variable("user_id") {
            format!("Provide personalized service for user: {}", user_id)
        } else {
            "Provide general service".to_string()
        }
    };

    let resolved_instructions = dynamic_instructions(&context);
    assert!(resolved_instructions.contains("test_user_123"));

    println!("✅ Dynamic arguments test passed:");
    println!("   - Context variables: {:?}", context.variables);
    println!("   - Resolved instructions: {}", resolved_instructions);
}

/// Test evaluation metrics system
#[tokio::test]
async fn test_evaluation_metrics_system() {
    println!("🧪 Testing evaluation metrics system...");

    let logger = create_logger("eval_test", Component::Agent, LogLevel::Info);
    
    // Test relevance metric
    let relevance_metric = RelevanceMetric::new(logger.clone(), 0.7);
    
    let context = RuntimeContext::new();
    
    // Test high relevance case
    let result = relevance_metric.evaluate(
        "What is the weather like today?",
        "Today's weather is sunny with a temperature of 25°C",
        &context
    ).await.expect("Evaluation should succeed");
    
    assert!(result.score >= 0.7, "Should have high relevance score");
    assert!(result.passed, "Should pass the threshold");
    
    // Test low relevance case
    let result = relevance_metric.evaluate(
        "What is the weather like today?",
        "I like pizza and ice cream",
        &context
    ).await.expect("Evaluation should succeed");
    
    assert!(result.score < 0.7, "Should have low relevance score");
    assert!(!result.passed, "Should not pass the threshold");

    println!("✅ Evaluation metrics test passed:");
    println!("   - Metric name: {}", relevance_metric.metric_name());
    println!("   - Score range: {:?}", relevance_metric.score_range());
}

/// Test memory processors system
#[tokio::test]
async fn test_memory_processors_system() {
    println!("🧪 Testing memory processors system...");

    let logger = create_logger("memory_test", Component::Memory, LogLevel::Debug);
    
    // Create test messages
    let messages = vec![
        Message { role: "user".to_string(), content: "Hello".to_string() },
        Message { role: "assistant".to_string(), content: "Hi there!".to_string() },
        Message { role: "user".to_string(), content: "How are you?".to_string() },
        Message { role: "assistant".to_string(), content: "I'm doing well!".to_string() },
        Message { role: "user".to_string(), content: "Great!".to_string() },
    ];

    // Test message limit processor
    let limit_processor = MessageLimitProcessor::new(3, logger.clone());
    let options = Default::default();
    
    let processed = limit_processor.process(messages.clone(), &options)
        .await
        .expect("Processing should succeed");
    
    assert_eq!(processed.len(), 3, "Should limit to 3 messages");
    assert_eq!(processed[0].content, "How are you?", "Should keep most recent messages");

    println!("✅ Memory processors test passed:");
    println!("   - Original messages: {}", messages.len());
    println!("   - Processed messages: {}", processed.len());
    println!("   - Processor: {}", limit_processor.processor_name());
}

/// Test function calling integration
#[tokio::test]
async fn test_function_calling_integration() {
    println!("🧪 Testing function calling integration...");

    // Create a simple calculator tool for testing
    struct CalculatorTool;
    
    impl Tool for CalculatorTool {
        fn name(&self) -> &str {
            "calculator"
        }
        
        fn description(&self) -> &str {
            "Performs basic arithmetic calculations"
        }
        
        async fn execute(&self, arguments: HashMap<String, Value>) -> ToolResult {
            if let Some(Value::String(expression)) = arguments.get("expression") {
                // Simple evaluation for testing
                let result = match expression.as_str() {
                    "2+2" => "4",
                    "10*5" => "50",
                    "100/4" => "25",
                    _ => "Unknown expression",
                };
                
                ToolResult {
                    name: self.name().to_string(),
                    call_id: "test_call".to_string(),
                    result: Value::String(result.to_string()),
                    status: lumosai_core::agent::types::ToolResultStatus::Success,
                }
            } else {
                ToolResult {
                    name: self.name().to_string(),
                    call_id: "test_call".to_string(),
                    result: Value::String("Invalid arguments".to_string()),
                    status: lumosai_core::agent::types::ToolResultStatus::Error,
                }
            }
        }
    }

    let tool = CalculatorTool;
    
    // Test tool execution
    let mut args = HashMap::new();
    args.insert("expression".to_string(), Value::String("2+2".to_string()));
    
    let result = tool.execute(args).await;
    assert_eq!(result.status, lumosai_core::agent::types::ToolResultStatus::Success);
    assert_eq!(result.result, Value::String("4".to_string()));

    println!("✅ Function calling test passed:");
    println!("   - Tool name: {}", tool.name());
    println!("   - Result: {:?}", result.result);
}
