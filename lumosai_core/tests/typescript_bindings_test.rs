use std::collections::HashMap;
use serde_json::Value;
use lumosai_core::bindings::{
    TypeScriptBindings, TSAgentConfig, TSMemoryConfig, TSToolDefinition, 
    TSParameterSchema, TSPropertySchema, BindingLanguage, generate_bindings
};

#[tokio::test]
async fn test_typescript_agent_creation() {
    let config = TSAgentConfig {
        name: "test-agent".to_string(),
        instructions: "You are a helpful assistant".to_string(),
        model: "gpt-4".to_string(),
        tools: vec!["calculator".to_string(), "web_search".to_string()],
        memory_config: Some(TSMemoryConfig {
            max_tokens: Some(2000),
            strategy: "sliding_window".to_string(),
            persistence: false,
        }),
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("version".to_string(), Value::String("1.0".to_string()));
            meta.insert("environment".to_string(), Value::String("test".to_string()));
            meta
        },
    };

    let result = TypeScriptBindings::create_agent(config).await;
    assert!(result.is_ok());
    
    let agent_id = result.unwrap();
    assert!(!agent_id.is_empty());
    println!("Created agent with ID: {}", agent_id);
}

#[tokio::test]
async fn test_typescript_agent_execution() {
    let agent_id = "test-agent-123";
    let message = "Hello, how can you help me?";
    let mut context = HashMap::new();
    context.insert("session_id".to_string(), Value::String("session-456".to_string()));
    context.insert("user_id".to_string(), Value::String("user-789".to_string()));

    let result = TypeScriptBindings::execute_agent(agent_id, message, Some(context)).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert!(!response.content.is_empty());
    assert!(response.content.contains(agent_id));
    assert!(response.content.contains(message));
    assert!(response.usage.is_some());
    
    let usage = response.usage.unwrap();
    assert!(usage.total_tokens > 0);
    assert_eq!(usage.total_tokens, usage.prompt_tokens + usage.completion_tokens);
    
    println!("Agent response: {}", response.content);
}

#[tokio::test]
async fn test_typescript_tool_registration() {
    let mut properties = HashMap::new();
    properties.insert("query".to_string(), TSPropertySchema {
        r#type: "string".to_string(),
        description: "The search query".to_string(),
        default: None,
        enum_values: None,
    });
    properties.insert("max_results".to_string(), TSPropertySchema {
        r#type: "number".to_string(),
        description: "Maximum number of results".to_string(),
        default: Some(Value::Number(serde_json::Number::from(10))),
        enum_values: None,
    });

    let tool_def = TSToolDefinition {
        name: "web_search".to_string(),
        description: "Search the web for information".to_string(),
        parameters: TSParameterSchema {
            r#type: "object".to_string(),
            properties,
            required: vec!["query".to_string()],
        },
        handler: r#"
            function webSearch(params) {
                const { query, max_results = 10 } = params;
                // Simulate web search
                return {
                    results: [
                        { title: "Result 1", url: "https://example.com/1", snippet: "First result" },
                        { title: "Result 2", url: "https://example.com/2", snippet: "Second result" }
                    ],
                    query: query,
                    total_results: 2
                };
            }
        "#.to_string(),
    };

    let result = TypeScriptBindings::register_tool(tool_def).await;
    assert!(result.is_ok());
    
    println!("Successfully registered web_search tool");
}

#[tokio::test]
async fn test_typescript_list_tools() {
    // First register a tool
    let tool_def = TSToolDefinition {
        name: "calculator".to_string(),
        description: "Perform mathematical calculations".to_string(),
        parameters: TSParameterSchema {
            r#type: "object".to_string(),
            properties: {
                let mut props = HashMap::new();
                props.insert("expression".to_string(), TSPropertySchema {
                    r#type: "string".to_string(),
                    description: "Mathematical expression to evaluate".to_string(),
                    default: None,
                    enum_values: None,
                });
                props
            },
            required: vec!["expression".to_string()],
        },
        handler: "function calculate(params) { return eval(params.expression); }".to_string(),
    };

    let _ = TypeScriptBindings::register_tool(tool_def).await;

    // Now list tools
    let result = TypeScriptBindings::list_tools().await;
    assert!(result.is_ok());
    
    let tools = result.unwrap();
    println!("Available tools: {}", tools.len());
    
    for tool in tools {
        println!("Tool: {} - {}", tool.name, tool.description);
    }
}

#[test]
fn test_error_formatting() {
    let error = lumosai_core::Error::Agent("Test agent error".to_string());
    let formatted = TypeScriptBindings::format_error(&error);
    
    assert_eq!(formatted["type"], "LumosError");
    assert_eq!(formatted["code"], "AGENT_ERROR");
    assert!(formatted["message"].as_str().unwrap().contains("Test agent error"));
    
    println!("Formatted error: {}", serde_json::to_string_pretty(&formatted).unwrap());
}

#[test]
fn test_type_definitions_generation() {
    let type_defs = TypeScriptBindings::generate_type_definitions();
    
    // Check that essential interfaces are present
    assert!(type_defs.contains("export interface AgentConfig"));
    assert!(type_defs.contains("export interface MemoryConfig"));
    assert!(type_defs.contains("export interface ToolDefinition"));
    assert!(type_defs.contains("export interface AgentResponse"));
    assert!(type_defs.contains("export class LumosClient"));
    
    // Check that methods are defined
    assert!(type_defs.contains("createAgent"));
    assert!(type_defs.contains("executeAgent"));
    assert!(type_defs.contains("registerTool"));
    assert!(type_defs.contains("listTools"));
    
    println!("Generated TypeScript definitions:");
    println!("{}", type_defs);
}

#[test]
fn test_binding_language_properties() {
    assert_eq!(BindingLanguage::TypeScript.extension(), "ts");
    assert_eq!(BindingLanguage::TypeScript.package_manager(), "npm");
    
    assert_eq!(BindingLanguage::Python.extension(), "py");
    assert_eq!(BindingLanguage::Python.package_manager(), "pip");
    
    assert_eq!(BindingLanguage::Go.extension(), "go");
    assert_eq!(BindingLanguage::Go.package_manager(), "go");
    
    assert_eq!(BindingLanguage::Java.extension(), "java");
    assert_eq!(BindingLanguage::Java.package_manager(), "maven");
}

#[test]
fn test_generate_bindings() {
    // Test TypeScript bindings generation
    let result = generate_bindings(BindingLanguage::TypeScript);
    assert!(result.is_ok());
    
    let bindings = result.unwrap();
    assert!(bindings.contains("export interface AgentConfig"));
    
    // Test unsupported language
    let result = generate_bindings(BindingLanguage::Python);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not yet implemented"));
}

#[test]
fn test_memory_config_strategies() {
    let sliding_window_config = TSMemoryConfig {
        max_tokens: Some(4000),
        strategy: "sliding_window".to_string(),
        persistence: true,
    };
    
    let summarization_config = TSMemoryConfig {
        max_tokens: Some(8000),
        strategy: "summarization".to_string(),
        persistence: false,
    };
    
    // Test serialization
    let json1 = serde_json::to_string(&sliding_window_config).unwrap();
    let json2 = serde_json::to_string(&summarization_config).unwrap();
    
    assert!(json1.contains("sliding_window"));
    assert!(json2.contains("summarization"));
    
    // Test deserialization
    let parsed1: TSMemoryConfig = serde_json::from_str(&json1).unwrap();
    let parsed2: TSMemoryConfig = serde_json::from_str(&json2).unwrap();
    
    assert_eq!(parsed1.strategy, "sliding_window");
    assert_eq!(parsed2.strategy, "summarization");
    assert!(parsed1.persistence);
    assert!(!parsed2.persistence);
}
