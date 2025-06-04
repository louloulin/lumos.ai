//! Tests for the simplified API improvements
//! 
//! These tests verify that the new builder patterns and simplified APIs
//! work correctly and provide better developer experience.

use lumosai_core::{Agent, Error, Tool};
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use lumosai_core::agent::{AgentBuilder};
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::tool::{ToolBuilder, create_tool};
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_agent_builder_basic_functionality() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello from agent!".to_string()]));
    
    let agent = AgentBuilder::new()
        .name("test_agent")
        .instructions("You are a helpful test assistant")
        .model(llm)
        .max_tool_calls(3)
        .tool_timeout(60)
        .enable_function_calling(true)
        .build()
        .expect("Failed to build agent");

    assert_eq!(agent.get_name(), "test_agent");
    assert_eq!(agent.get_instructions(), "You are a helpful test assistant");
    assert_eq!(agent.get_tools().len(), 0);
}

#[tokio::test]
async fn test_agent_builder_with_tools() {
    let llm = Arc::new(MockLlmProvider::new(vec![
        "I'll use the echo tool to help you.".to_string(),
        "Tool result: Echo: Hello World!".to_string(),
    ]));
    
    // Create a tool using the builder
    let echo_tool = ToolBuilder::new()
        .name("echo")
        .description("Echo a message back")
        .parameter("message", "string", "Message to echo", true)
        .handler(|params| {
            let message = params.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("No message");
            Ok(json!({"echo": message}))
        })
        .build()
        .expect("Failed to build tool");

    let agent = AgentBuilder::new()
        .name("echo_agent")
        .instructions("You are an echo assistant")
        .model(llm)
        .tool(Box::new(echo_tool))
        .build()
        .expect("Failed to build agent");

    assert_eq!(agent.get_name(), "echo_agent");
    assert_eq!(agent.get_tools().len(), 1);
    assert!(agent.get_tool("echo").is_some());
}

#[tokio::test]
async fn test_tool_builder_basic_functionality() {
    let tool = ToolBuilder::new()
        .name("calculator")
        .description("Performs basic math operations")
        .parameter("a", "number", "First number", true)
        .parameter("b", "number", "Second number", true)
        .parameter("operation", "string", "Operation (+, -, *, /)", true)
        .handler(|params| {
            let a = params.get("a").and_then(|v| v.as_f64()).ok_or_else(|| Error::Configuration("Invalid number a".to_string()))?;
            let b = params.get("b").and_then(|v| v.as_f64()).ok_or_else(|| Error::Configuration("Invalid number b".to_string()))?;
            let op = params.get("operation").and_then(|v| v.as_str()).ok_or_else(|| Error::Configuration("Invalid operation".to_string()))?;

            let result = match op {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => {
                    if b == 0.0 {
                        return Err(Error::Configuration("Division by zero".to_string()));
                    }
                    a / b
                },
                _ => return Err(Error::Configuration("Unknown operation".to_string())),
            };

            Ok(json!({"result": result}))
        })
        .build()
        .expect("Failed to build calculator tool");

    assert_eq!(tool.id(), "calculator");
    assert_eq!(tool.description(), "Performs basic math operations");
}

#[tokio::test]
async fn test_create_tool_convenience_function() {
    let tool = create_tool(
        "greet",
        "Greet a person",
        vec![
            ("name", "string", "Person's name", true),
            ("formal", "boolean", "Use formal greeting", false),
        ],
        |params| {
            let name = params.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Anonymous");
            let formal = params.get("formal")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let greeting = if formal {
                format!("Good day, {}", name)
            } else {
                format!("Hi, {}!", name)
            };
            
            Ok(json!({"greeting": greeting}))
        }
    ).expect("Failed to create greeting tool");

    assert_eq!(tool.id(), "greet");
    assert_eq!(tool.description(), "Greet a person");
}

#[tokio::test]
async fn test_agent_builder_validation() {
    // Test missing name
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    let result = AgentBuilder::new()
        .instructions("Test instructions")
        .model(llm.clone())
        .build();
    assert!(result.is_err());

    // Test missing instructions
    let result = AgentBuilder::new()
        .name("test")
        .model(llm.clone())
        .build();
    assert!(result.is_err());

    // Test missing model
    let result = AgentBuilder::new()
        .name("test")
        .instructions("Test instructions")
        .build();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_builder_validation() {
    // Test missing name
    let result = ToolBuilder::new()
        .description("Test description")
        .handler(|_| Ok(json!({})))
        .build();
    assert!(result.is_err());

    // Test missing description
    let result = ToolBuilder::new()
        .name("test")
        .handler(|_| Ok(json!({})))
        .build();
    assert!(result.is_err());

    // Test missing handler
    let result = ToolBuilder::new()
        .name("test")
        .description("Test description")
        .build();
    assert!(result.is_err());
}

#[tokio::test]
async fn test_agent_generation_with_simplified_api() {
    let llm = Arc::new(MockLlmProvider::new(vec![
        "I'll help you calculate that using the calculator tool.".to_string(),
    ]));
    
    // Create calculator tool
    let calc_tool = create_tool(
        "calculator",
        "Performs basic math operations",
        vec![
            ("a", "number", "First number", true),
            ("b", "number", "Second number", true),
            ("operation", "string", "Operation", true),
        ],
        |params| {
            let a = params.get("a").and_then(|v| v.as_f64()).ok_or_else(|| Error::Configuration("Invalid number a".to_string()))?;
            let b = params.get("b").and_then(|v| v.as_f64()).ok_or_else(|| Error::Configuration("Invalid number b".to_string()))?;
            let op = params.get("operation").and_then(|v| v.as_str()).ok_or_else(|| Error::Configuration("Invalid operation".to_string()))?;

            let result = match op {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => a / b,
                _ => return Err(Error::Configuration("Unknown operation".to_string())),
            };

            Ok(json!({"result": result}))
        }
    ).expect("Failed to create calculator tool");

    let agent = AgentBuilder::new()
        .name("math_agent")
        .instructions("You are a math assistant that can perform calculations")
        .model(llm)
        .tool(Box::new(calc_tool))
        .build()
        .expect("Failed to build agent");

    let message = Message {
        role: Role::User,
        content: "Calculate 5 + 3".to_string(),
        metadata: None,
        name: None,
    };

    let result = agent.generate(&[message], &AgentGenerateOptions::default()).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(!response.response.is_empty());
}

#[tokio::test]
async fn test_performance_comparison() {
    // Test that the new API doesn't significantly impact performance
    let llm = Arc::new(MockLlmProvider::new(vec!["Quick response".to_string()]));
    
    // Measure agent creation time
    let start = Instant::now();
    let _agent = AgentBuilder::new()
        .name("perf_test")
        .instructions("Performance test agent")
        .model(llm)
        .build()
        .expect("Failed to build agent");
    let creation_time = start.elapsed();
    
    // Agent creation should be fast (< 1ms for simple cases)
    assert!(creation_time.as_millis() < 10, "Agent creation took too long: {:?}", creation_time);
}

#[tokio::test]
async fn test_metadata_and_context() {
    let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
    
    let agent = AgentBuilder::new()
        .name("metadata_test")
        .instructions("Test agent with metadata")
        .model(llm)
        .add_metadata("version", "1.0")
        .add_metadata("category", "test")
        .add_context("environment", json!("test"))
        .add_context("debug", json!(true))
        .build()
        .expect("Failed to build agent");

    assert_eq!(agent.get_name(), "metadata_test");
    // Note: We can't directly test metadata/context access without extending the Agent trait
    // This test mainly ensures the builder accepts these values without error
}

#[tokio::test]
async fn test_tool_with_default_parameters() {
    let tool = ToolBuilder::new()
        .name("greet_with_default")
        .description("Greet with optional formality")
        .parameter("name", "string", "Person's name", true)
        .parameter_with_default("formal", "boolean", "Use formal greeting", false, json!(false))
        .handler(|params| {
            let name = params.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Anonymous");
            let formal = params.get("formal")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let greeting = if formal {
                format!("Good day, {}", name)
            } else {
                format!("Hi, {}!", name)
            };
            
            Ok(json!({"greeting": greeting}))
        })
        .build()
        .expect("Failed to build tool with defaults");

    assert_eq!(tool.id(), "greet_with_default");
}

#[tokio::test]
async fn test_multiple_tools_with_agent() {
    let llm = Arc::new(MockLlmProvider::new(vec!["I can help with both math and greetings!".to_string()]));
    
    let calc_tool = create_tool(
        "calc",
        "Calculator",
        vec![("expr", "string", "Math expression", true)],
        |_| Ok(json!({"result": 42}))
    ).expect("Failed to create calc tool");
    
    let greet_tool = create_tool(
        "greet",
        "Greeter",
        vec![("name", "string", "Name", true)],
        |params| {
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("World");
            Ok(json!({"greeting": format!("Hello, {}!", name)}))
        }
    ).expect("Failed to create greet tool");

    let agent = AgentBuilder::new()
        .name("multi_tool_agent")
        .instructions("I can do math and greetings")
        .model(llm)
        .tools(vec![Box::new(calc_tool) as Box<dyn Tool>, Box::new(greet_tool) as Box<dyn Tool>])
        .build()
        .expect("Failed to build multi-tool agent");

    assert_eq!(agent.get_tools().len(), 2);
    assert!(agent.get_tool("calc").is_some());
    assert!(agent.get_tool("greet").is_some());
}
