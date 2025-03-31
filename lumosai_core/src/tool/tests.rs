use serde_json::json;
use tokio::sync::watch;

use crate::tool::{
    Tool, GenericTool, FunctionTool, ToolSchema, ToolExecutionContext, 
    ToolExecutionOptions, ParameterSchema
};

#[tokio::test]
async fn test_generic_tool() {
    // Create a schema for a simple calculator tool
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "a".to_string(),
            description: "First number".to_string(),
            r#type: "number".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "b".to_string(),
            description: "Second number".to_string(),
            r#type: "number".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "operation".to_string(),
            description: "Operation to perform".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);
    
    // Create a calculator tool
    let calculator = GenericTool::new(
        "calculator",
        "Performs basic arithmetic operations",
        schema,
        |params, _context| {
            let a = params["a"].as_f64().unwrap_or(0.0);
            let b = params["b"].as_f64().unwrap_or(0.0);
            let op = params["operation"].as_str().unwrap_or("add");
            
            let result = match op {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(crate::error::Error::Tool("Cannot divide by zero".to_string()));
                    }
                    a / b
                }
                _ => return Err(crate::error::Error::Tool(format!("Unknown operation: {}", op))),
            };
            
            Ok(json!(result))
        },
    );
    
    // Create context and options for execution
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::with_validation();
    
    // Test addition
    let params = json!({
        "a": 5,
        "b": 3,
        "operation": "add"
    });
    let result = calculator.execute(params, context.clone(), &options).await.unwrap();
    assert_eq!(result, json!(8.0));
    
    // Test division
    let params = json!({
        "a": 10,
        "b": 2,
        "operation": "divide"
    });
    let result = calculator.execute(params, context.clone(), &options).await.unwrap();
    assert_eq!(result, json!(5.0));
    
    // Test division by zero (should fail)
    let params = json!({
        "a": 10,
        "b": 0,
        "operation": "divide"
    });
    let err = calculator.execute(params, context.clone(), &options).await.unwrap_err();
    assert!(matches!(err, crate::error::Error::Tool(_)));
    
    // Test missing required parameter (should fail validation)
    let params = json!({
        "a": 5,
        "operation": "add"
    });
    let err = calculator.execute(params, context, &options).await.unwrap_err();
    assert!(matches!(err, crate::error::Error::InvalidParams(_)));
}

#[tokio::test]
async fn test_tool_with_json_schema() {
    // Create a tool with JSON Schema
    let schema = ToolSchema::with_json_schema(json!({
        "type": "object",
        "required": ["text"],
        "properties": {
            "text": {
                "type": "string",
                "description": "Text to analyze"
            },
            "language": {
                "type": "string",
                "enum": ["en", "es", "fr"],
                "default": "en",
                "description": "Language code"
            }
        }
    }));
    
    // Create a text analysis tool
    let text_analyzer = GenericTool::new(
        "text-analyzer",
        "Analyzes text and returns statistics",
        schema,
        |params, _context| {
            let text = params["text"].as_str().unwrap_or("");
            let lang = params["language"].as_str().unwrap_or("en");
            
            let word_count = text.split_whitespace().count();
            let char_count = text.chars().count();
            
            Ok(json!({
                "wordCount": word_count,
                "charCount": char_count,
                "language": lang
            }))
        },
    );
    
    // Create context and options for execution
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::default();
    
    // Test analysis
    let params = json!({
        "text": "Hello world, this is a test.",
        "language": "en"
    });
    let result = text_analyzer.execute(params, context, &options).await.unwrap();
    
    assert_eq!(result["wordCount"], json!(6));
    assert_eq!(result["charCount"], json!(28));
    assert_eq!(result["language"], json!("en"));
}

#[tokio::test]
async fn test_tool_execution_context_abort() {
    // Create a tool that simulates a long-running operation
    let slow_tool = FunctionTool::new(
        "slow-tool",
        "A tool that simulates a long-running operation",
        ToolSchema::new(vec![]),
        |_params| {
            // This would normally be a long operation, but we'll just return immediately
            // since we're testing the abort signal
            Ok(json!({ "status": "completed" }))
        },
    );
    
    // Create a cancel channel
    let (tx, rx) = watch::channel(false);
    
    // Create a context with the abort signal
    let context = ToolExecutionContext::new().with_abort_signal(rx);
    
    // Trigger an abort
    tx.send(true).unwrap();
    
    // The tool should abort without executing
    let options = ToolExecutionOptions::default();
    let result = slow_tool.execute(json!({}), context, &options).await;
    
    assert!(result.is_err());
    if let Err(err) = result {
        if let crate::error::Error::Tool(msg) = err {
            assert_eq!(msg, "Tool execution aborted");
        } else {
            panic!("Expected Tool error, got: {:?}", err);
        }
    }
}

#[tokio::test]
async fn test_tool_with_output_validation() {
    // Create a tool with output schema for validation
    let schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "name".to_string(),
            description: "Person's name".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
        ParameterSchema {
            name: "age".to_string(),
            description: "Person's age".to_string(),
            r#type: "number".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);
    
    // Output schema that requires a specific format
    let output_schema = json!({
        "type": "object",
        "required": ["fullName", "ageInYears", "isAdult"],
        "properties": {
            "fullName": { "type": "string" },
            "ageInYears": { "type": "number" },
            "isAdult": { "type": "boolean" }
        }
    });
    
    // Create a person formatter tool
    let person_formatter = GenericTool::new(
        "person-formatter",
        "Formats person information",
        schema,
        |params, _context| {
            let name = params["name"].as_str().unwrap_or("");
            let age = params["age"].as_f64().unwrap_or(0.0) as u32;
            
            // This would normally produce a properly formatted output
            // but we're intentionally returning an incorrect format when age < 10
            if age < 10 {
                // Missing isAdult field (would fail validation)
                return Ok(json!({
                    "fullName": name,
                    "ageInYears": age
                }));
            }
            
            Ok(json!({
                "fullName": name,
                "ageInYears": age,
                "isAdult": age >= 18
            }))
        },
    ).with_output_schema(output_schema);
    
    // Create context and options with validation
    let context = ToolExecutionContext::new();
    let options = ToolExecutionOptions::with_validation();
    
    // Test with valid output (adult)
    let params = json!({
        "name": "John Doe",
        "age": 30
    });
    let result = person_formatter.execute(params, context.clone(), &options).await.unwrap();
    assert_eq!(result["fullName"], json!("John Doe"));
    assert_eq!(result["ageInYears"], json!(30));
    assert_eq!(result["isAdult"], json!(true));
    
    // Test with valid output (child but proper format)
    let params = json!({
        "name": "Billy Kid",
        "age": 12
    });
    let result = person_formatter.execute(params, context.clone(), &options).await.unwrap();
    assert_eq!(result["isAdult"], json!(false));
    
    // Test with invalid output (missing required field)
    let params = json!({
        "name": "Baby Jane",
        "age": 3
    });
    
    // With validation, this should fail
    let err = person_formatter.execute(params.clone(), context.clone(), &options).await.unwrap_err();
    assert!(matches!(err, crate::error::Error::ValidationError(_)));
    
    // Without validation, it should pass
    let options_no_validation = ToolExecutionOptions::default();
    let result = person_formatter.execute(params, context, &options_no_validation).await.unwrap();
    assert_eq!(result["fullName"], json!("Baby Jane"));
    assert_eq!(result["ageInYears"], json!(3));
    assert!(!result.as_object().unwrap().contains_key("isAdult"));
} 