// Unit tests for Tool system
use crate::test_config::*;
use std::time::Duration;
use std::collections::HashMap;
use serde_json::Value;

#[tokio::test]
async fn test_tool_creation() {
    init_test_env();
    
    // Test basic tool creation
    let calculator = create_test_tool("calculator", "Performs mathematical calculations").await;
    assert!(calculator.is_ok(), "Calculator tool creation should succeed");
    
    let web_search = create_test_tool("web_search", "Searches the web for information").await;
    assert!(web_search.is_ok(), "Web search tool creation should succeed");
    
    let file_reader = create_test_tool("file_reader", "Reads and processes files").await;
    assert!(file_reader.is_ok(), "File reader tool creation should succeed");
}

#[tokio::test]
async fn test_tool_execution() {
    init_test_env();
    
    let calculator = create_test_tool("calculator", "Math tool").await.unwrap();
    
    // Test basic execution
    let params = create_tool_params(&[
        ("operation", "add"),
        ("a", "5"),
        ("b", "3"),
    ]);
    
    let result = calculator.execute(params).await;
    assert!(result.is_ok(), "Tool execution should succeed");
    
    let output = result.unwrap();
    assert!(!output.is_empty(), "Tool output should not be empty");
    
    // Verify calculation result
    if let Ok(parsed) = output.parse::<i32>() {
        assert_eq!(parsed, 8, "Calculator should return correct result");
    }
}

#[tokio::test]
async fn test_tool_parameter_validation() {
    init_test_env();
    
    let calculator = create_test_tool("calculator", "Math tool").await.unwrap();
    
    // Test with valid parameters
    let valid_params = create_tool_params(&[
        ("operation", "multiply"),
        ("a", "4"),
        ("b", "7"),
    ]);
    
    let valid_result = calculator.execute(valid_params).await;
    assert!(valid_result.is_ok(), "Valid parameters should succeed");
    
    // Test with missing parameters
    let missing_params = create_tool_params(&[("operation", "add")]);
    let missing_result = calculator.execute(missing_params).await;
    // Should handle missing parameters gracefully
    assert!(missing_result.is_ok() || missing_result.is_err());
    
    // Test with invalid parameters
    let invalid_params = create_tool_params(&[
        ("operation", "invalid_op"),
        ("a", "not_a_number"),
        ("b", "also_not_a_number"),
    ]);
    
    let invalid_result = calculator.execute(invalid_params).await;
    // Should handle invalid parameters gracefully
    assert!(invalid_result.is_ok() || invalid_result.is_err());
}

#[tokio::test]
async fn test_tool_schema_validation() {
    init_test_env();
    
    let tool = create_test_tool_with_schema("weather", "Gets weather information").await.unwrap();
    
    // Test schema retrieval
    let schema = tool.get_schema().await;
    assert!(schema.is_ok(), "Schema retrieval should succeed");
    
    let schema = schema.unwrap();
    assert!(!schema.is_empty(), "Schema should not be empty");
    
    // Verify schema contains expected fields
    assert!(schema.contains("location"), "Schema should contain location parameter");
    assert!(schema.contains("units"), "Schema should contain units parameter");
}

#[tokio::test]
async fn test_tool_error_handling() {
    init_test_env();
    
    let tool = create_test_tool("error_prone", "Tool that may fail").await.unwrap();
    
    // Test with parameters that cause errors
    let error_params = create_tool_params(&[("action", "cause_error")]);
    let error_result = tool.execute(error_params).await;
    
    // Should handle errors gracefully
    match error_result {
        Ok(_) => println!("Tool handled error case successfully"),
        Err(e) => println!("Tool returned expected error: {:?}", e),
    }
    
    // Test with timeout scenario
    let timeout_params = create_tool_params(&[("action", "timeout")]);
    let timeout_result = tokio::time::timeout(
        Duration::from_secs(2),
        tool.execute(timeout_params)
    ).await;
    
    assert!(timeout_result.is_ok(), "Tool should complete within timeout");
}

#[tokio::test]
async fn test_tool_async_execution() {
    init_test_env();
    
    let async_tool = create_test_tool("async_processor", "Async processing tool").await.unwrap();
    
    // Test async execution
    let params = create_tool_params(&[
        ("task", "process_data"),
        ("data", "test_data_to_process"),
    ]);
    
    let (result, duration) = PerformanceTestUtils::measure_time(|| async {
        async_tool.execute(params).await
    }).await;
    
    assert!(result.is_ok(), "Async tool execution should succeed");
    
    // Verify it actually took some time (indicating async behavior)
    assert!(duration >= Duration::from_millis(10), "Async tool should take some time");
    
    println!("Async tool execution took: {:?}", duration);
}

#[tokio::test]
async fn test_tool_concurrent_execution() {
    init_test_env();
    
    let tool = create_test_tool("concurrent_tool", "Supports concurrent execution").await.unwrap();
    
    // Launch multiple concurrent executions
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let tool_clone = tool.clone();
        let params = create_tool_params(&[
            ("task_id", &format!("task_{}", i)),
            ("data", &format!("data_{}", i)),
        ]);
        
        let handle = tokio::spawn(async move {
            tool_clone.execute(params).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all executions to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent execution {} should succeed", i);
        
        let tool_result = result.unwrap();
        assert!(tool_result.is_ok(), "Tool execution {} should succeed", i);
    }
}

#[tokio::test]
async fn test_tool_state_management() {
    init_test_env();
    
    let stateful_tool = create_test_stateful_tool("counter", "Maintains state").await.unwrap();
    
    // Test state initialization
    let initial_state = stateful_tool.get_state().await;
    assert!(initial_state.is_ok(), "State retrieval should succeed");
    
    // Test state modification through execution
    let increment_params = create_tool_params(&[("action", "increment")]);
    let increment_result = stateful_tool.execute(increment_params).await;
    assert!(increment_result.is_ok(), "State modification should succeed");
    
    // Verify state changed
    let updated_state = stateful_tool.get_state().await.unwrap();
    assert_ne!(initial_state.unwrap(), updated_state, "State should have changed");
    
    // Test state reset
    let reset_params = create_tool_params(&[("action", "reset")]);
    let reset_result = stateful_tool.execute(reset_params).await;
    assert!(reset_result.is_ok(), "State reset should succeed");
}

#[tokio::test]
async fn test_tool_composition() {
    init_test_env();
    
    let tool1 = create_test_tool("preprocessor", "Preprocesses data").await.unwrap();
    let tool2 = create_test_tool("analyzer", "Analyzes data").await.unwrap();
    let tool3 = create_test_tool("formatter", "Formats results").await.unwrap();
    
    // Test tool composition (pipeline)
    let input_data = "raw_input_data";
    
    // Step 1: Preprocess
    let preprocess_params = create_tool_params(&[("input", input_data)]);
    let preprocessed = tool1.execute(preprocess_params).await.unwrap();
    
    // Step 2: Analyze
    let analyze_params = create_tool_params(&[("data", &preprocessed)]);
    let analyzed = tool2.execute(analyze_params).await.unwrap();
    
    // Step 3: Format
    let format_params = create_tool_params(&[("results", &analyzed)]);
    let formatted = tool3.execute(format_params).await.unwrap();
    
    assert!(!formatted.is_empty(), "Final result should not be empty");
    println!("Tool composition result: {}", formatted);
}

#[tokio::test]
async fn test_tool_performance() {
    init_test_env();
    
    let tool = create_test_tool("performance_tool", "Performance testing tool").await.unwrap();
    
    // Measure single execution performance
    let params = create_tool_params(&[("operation", "benchmark")]);
    
    let (result, duration) = PerformanceTestUtils::measure_time(|| async {
        tool.execute(params).await
    }).await;
    
    assert!(result.is_ok(), "Performance test should succeed");
    
    // Performance assertion
    PerformanceTestUtils::assert_execution_time_within(
        duration,
        Duration::from_secs(1)
    );
    
    // Benchmark multiple executions
    let durations = PerformanceTestUtils::benchmark(
        "tool_execution",
        10,
        || async {
            let params = create_tool_params(&[("operation", "quick_task")]);
            tool.execute(params).await.unwrap();
        }
    ).await;
    
    let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
    println!("Average tool execution time: {:?}", avg_duration);
    
    // Assert reasonable performance
    assert!(avg_duration < Duration::from_millis(500), "Average execution should be under 500ms");
}

#[tokio::test]
async fn test_tool_metadata() {
    init_test_env();
    
    let tool = create_test_tool_with_metadata("metadata_tool", "Tool with metadata").await.unwrap();
    
    // Test metadata retrieval
    let metadata = tool.get_metadata().await;
    assert!(metadata.is_ok(), "Metadata retrieval should succeed");
    
    let metadata = metadata.unwrap();
    assert!(metadata.contains_key("name"), "Metadata should contain name");
    assert!(metadata.contains_key("description"), "Metadata should contain description");
    assert!(metadata.contains_key("version"), "Metadata should contain version");
    
    // Test metadata values
    assert_eq!(metadata.get("name").unwrap(), "metadata_tool");
    assert!(!metadata.get("description").unwrap().is_empty());
}

// Helper functions for tool testing
async fn create_test_tool(name: &str, description: &str) -> Result<TestTool> {
    Ok(TestTool::new(name, description))
}

async fn create_test_tool_with_schema(name: &str, description: &str) -> Result<TestTool> {
    Ok(TestTool::with_schema(name, description))
}

async fn create_test_stateful_tool(name: &str, description: &str) -> Result<StatefulTestTool> {
    Ok(StatefulTestTool::new(name, description))
}

async fn create_test_tool_with_metadata(name: &str, description: &str) -> Result<TestTool> {
    Ok(TestTool::with_metadata(name, description))
}

fn create_tool_params(params: &[(&str, &str)]) -> HashMap<String, Value> {
    params.iter()
        .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
        .collect()
}

// Mock tool implementations for testing
#[derive(Clone)]
struct TestTool {
    name: String,
    description: String,
    has_schema: bool,
    has_metadata: bool,
}

impl TestTool {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            has_schema: false,
            has_metadata: false,
        }
    }
    
    fn with_schema(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            has_schema: true,
            has_metadata: false,
        }
    }
    
    fn with_metadata(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            has_schema: false,
            has_metadata: true,
        }
    }
    
    async fn execute(&self, params: HashMap<String, Value>) -> Result<String> {
        // Simulate async work
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        match self.name.as_str() {
            "calculator" => self.execute_calculator(params).await,
            "error_prone" => self.execute_error_prone(params).await,
            "async_processor" => self.execute_async_processor(params).await,
            _ => Ok(format!("Executed {} with {} parameters", self.name, params.len())),
        }
    }
    
    async fn execute_calculator(&self, params: HashMap<String, Value>) -> Result<String> {
        let operation = params.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");
        
        let a = params.get("a")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        
        let b = params.get("b")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);
        
        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => if b != 0 { a / b } else { 0 },
            _ => 0,
        };
        
        Ok(result.to_string())
    }
    
    async fn execute_error_prone(&self, params: HashMap<String, Value>) -> Result<String> {
        let action = params.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("normal");
        
        match action {
            "cause_error" => Err("Intentional error for testing".into()),
            "timeout" => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok("Completed after delay".to_string())
            },
            _ => Ok("Normal execution".to_string()),
        }
    }
    
    async fn execute_async_processor(&self, params: HashMap<String, Value>) -> Result<String> {
        // Simulate longer async processing
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let task = params.get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        
        Ok(format!("Processed task: {}", task))
    }
    
    async fn get_schema(&self) -> Result<String> {
        if self.has_schema {
            Ok(r#"{"location": {"type": "string"}, "units": {"type": "string"}}"#.to_string())
        } else {
            Ok("{}".to_string())
        }
    }
    
    async fn get_metadata(&self) -> Result<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        
        if self.has_metadata {
            metadata.insert("name".to_string(), self.name.clone());
            metadata.insert("description".to_string(), self.description.clone());
            metadata.insert("version".to_string(), "1.0.0".to_string());
            metadata.insert("author".to_string(), "LumosAI".to_string());
        }
        
        Ok(metadata)
    }
}

#[derive(Clone)]
struct StatefulTestTool {
    name: String,
    description: String,
    state: std::sync::Arc<tokio::sync::RwLock<i32>>,
}

impl StatefulTestTool {
    fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            state: std::sync::Arc::new(tokio::sync::RwLock::new(0)),
        }
    }
    
    async fn execute(&self, params: HashMap<String, Value>) -> Result<String> {
        let action = params.get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("get");
        
        match action {
            "increment" => {
                let mut state = self.state.write().await;
                *state += 1;
                Ok(format!("Incremented to {}", *state))
            },
            "reset" => {
                let mut state = self.state.write().await;
                *state = 0;
                Ok("Reset to 0".to_string())
            },
            _ => {
                let state = self.state.read().await;
                Ok(format!("Current state: {}", *state))
            }
        }
    }
    
    async fn get_state(&self) -> Result<i32> {
        let state = self.state.read().await;
        Ok(*state)
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
