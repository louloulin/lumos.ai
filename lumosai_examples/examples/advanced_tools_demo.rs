//! Advanced Tools Demo
//!
//! This example demonstrates the use of advanced tools with Function Calling
//! capabilities, including file management and data analysis tools.

use std::collections::HashMap;
use serde_json::json;
use tokio;
use futures::stream::BoxStream;

use lumosai_core::{
    base::{BaseComponent, ComponentConfig},
    error::Result,
    logger::{Component, LogLevel, ConsoleLogger},
    llm::{
        LlmProvider, LlmOptions, Message, Role,
        function_calling::{FunctionDefinition, FunctionCall, ToolChoice},
        provider::FunctionCallingResponse,
    },
    tool::{Tool, ToolExecutionContext, ToolExecutionOptions},
};

use lumosai_examples::advanced_tools::{FileManagerTool, DataAnalysisTool};

/// Mock LLM provider for demonstration
struct MockAdvancedLLM;

#[async_trait::async_trait]
impl LlmProvider for MockAdvancedLLM {
    async fn generate(&self, prompt: &str, _options: &LlmOptions) -> Result<String> {
        if prompt.contains("analyze") && prompt.contains("data") {
            Ok("I'll analyze the data for outliers.".to_string())
        } else if prompt.contains("file") || prompt.contains("directory") {
            Ok("I'll list the contents of the examples directory.".to_string())
        } else {
            Ok("I understand. How can I help you with file management or data analysis?".to_string())
        }
    }

    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let last_message = messages.last()
            .map(|m| m.content.as_str())
            .unwrap_or("");
        self.generate(last_message, options).await
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        let response = self.generate(prompt, options).await?;
        let stream = futures::stream::once(async move { Ok(response) });
        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3, 0.4, 0.5])
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        _functions: &[FunctionDefinition],
        _tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        let last_message = messages.last()
            .map(|m| m.content.as_str())
            .unwrap_or("");

        if last_message.contains("analyze") && last_message.contains("data") {
            // Simulate calling data analysis tool
            let function_call = FunctionCall {
                id: Some("call_data_analysis".to_string()),
                name: "data_analysis".to_string(),
                arguments: json!({
                    "data": [1.0, 2.0, 3.0, 4.0, 5.0, 100.0, 6.0, 7.0, 8.0, 9.0],
                    "analysis_type": "outliers"
                }).to_string(),
            };

            Ok(FunctionCallingResponse {
                content: Some("I'll analyze the data for outliers.".to_string()),
                function_calls: vec![function_call],
                finish_reason: "function_call".to_string(),
            })
        } else if last_message.contains("file") || last_message.contains("directory") {
            // Simulate calling file management tool
            let function_call = FunctionCall {
                id: Some("call_file_manager".to_string()),
                name: "file_manager".to_string(),
                arguments: json!({
                    "operation": "list_directory",
                    "path": "./examples"
                }).to_string(),
            };

            Ok(FunctionCallingResponse {
                content: Some("I'll list the contents of the examples directory.".to_string()),
                function_calls: vec![function_call],
                finish_reason: "function_call".to_string(),
            })
        } else {
            let response = self.generate_with_messages(messages, options).await?;
            Ok(FunctionCallingResponse {
                content: Some(response),
                function_calls: Vec::new(),
                finish_reason: "stop".to_string(),
            })
        }
    }
}

/// Advanced agent with sophisticated tools
struct AdvancedAgent {
    base: BaseComponent,
    llm: Box<dyn LlmProvider>,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl AdvancedAgent {
    fn new() -> Self {
        let component_config = ComponentConfig {
            name: Some("AdvancedAgent".to_string()),
            component: Component::Agent,
            log_level: Some(LogLevel::Info),
        };

        let mut tools: HashMap<String, Box<dyn Tool>> = HashMap::new();
        
        // Add file manager tool with restricted access
        let file_manager = FileManagerTool::new(vec![
            "./examples".to_string(),
            "./test_data".to_string(),
        ]);
        tools.insert("file_manager".to_string(), Box::new(file_manager));
        
        // Add data analysis tool
        let data_analysis = DataAnalysisTool::new();
        tools.insert("data_analysis".to_string(), Box::new(data_analysis));

        Self {
            base: BaseComponent::new(component_config),
            llm: Box::new(MockAdvancedLLM),
            tools,
        }
    }

    async fn process_request(&self, prompt: &str) -> Result<String> {
        println!("🤖 Processing request: {}", prompt);

        // Create messages
        let messages = vec![Message {
            role: Role::User,
            content: prompt.to_string(),
            name: None,
            metadata: None,
        }];

        // Get function definitions from tools
        let function_definitions: Vec<FunctionDefinition> = self.tools.values()
            .map(|tool| FunctionDefinition::from_tool(tool.as_ref()))
            .collect();

        // Create LLM options
        let options = LlmOptions {
            model: Some("mock-advanced".to_string()),
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stream: false,
            stop: None,
            extra: serde_json::Map::new(),
        };

        // Generate response with function calling
        let response = self.llm.generate_with_functions(
            &messages,
            &function_definitions,
            &ToolChoice::Auto,
            &options,
        ).await?;

        println!("💭 LLM Response: {:?}", response.content);

        // Execute function calls if any
        if !response.function_calls.is_empty() {
            for function_call in response.function_calls {
                println!("🔧 Executing function: {} with args: {}",
                    function_call.name, function_call.arguments);

                if let Some(tool) = self.tools.get(&function_call.name) {
                    let params = function_call.parse_arguments()?;
                    let context = ToolExecutionContext::default();
                    let options = ToolExecutionOptions::default();

                    match tool.execute(params, context, &options).await {
                        Ok(result) => {
                            println!("✅ Function result: {}", serde_json::to_string_pretty(&result)?);
                            return Ok(format!("{}\n\nFunction Result:\n{}",
                                response.content.unwrap_or_default(),
                                serde_json::to_string_pretty(&result)?));
                        },
                        Err(e) => {
                            println!("❌ Function execution failed: {}", e);
                            return Ok(format!("{}\n\nFunction execution failed: {}",
                                response.content.unwrap_or_default(), e));
                        }
                    }
                } else {
                    println!("❌ Unknown function: {}", function_call.name);
                }
            }
        }

        Ok(response.content.unwrap_or_default())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Advanced Tools Demo Starting...\n");

    // Initialize logger
    let _logger = std::sync::Arc::new(ConsoleLogger::new(
        "AdvancedToolsDemo",
        Component::Agent,
        LogLevel::Info
    ));
    
    // Create advanced agent
    let agent = AdvancedAgent::new();

    // Demo 1: Data Analysis
    println!("📊 Demo 1: Data Analysis");
    println!("{}", "=".repeat(50));
    
    let data_analysis_request = "Please analyze this data for outliers: [1, 2, 3, 4, 5, 100, 6, 7, 8, 9]";
    let result1 = agent.process_request(data_analysis_request).await?;
    println!("Result: {}\n", result1);

    // Demo 2: File Management
    println!("📁 Demo 2: File Management");
    println!("{}", "=".repeat(50));
    
    let file_management_request = "Can you list the contents of the examples directory?";
    let result2 = agent.process_request(file_management_request).await?;
    println!("Result: {}\n", result2);

    // Demo 3: Statistical Analysis
    println!("📈 Demo 3: Statistical Analysis");
    println!("{}", "=".repeat(50));
    
    // First, let's create some test data
    std::fs::create_dir_all("./test_data")?;
    
    let statistical_request = "I need to analyze some sales data for basic statistics";
    let result3 = agent.process_request(statistical_request).await?;
    println!("Result: {}\n", result3);

    // Demo 4: Complex Function Calling
    println!("🔧 Demo 4: Function Calling Capabilities");
    println!("{}", "=".repeat(50));
    
    // Demonstrate function definition creation
    let file_tool = FileManagerTool::new(vec!["./examples".to_string()]);
    let function_def = FunctionDefinition::from_tool(&file_tool);
    
    println!("Function Definition for File Manager:");
    println!("{}", serde_json::to_string_pretty(&function_def)?);
    
    // Demonstrate function call validation
    let test_call = FunctionCall {
        id: Some("test_call".to_string()),
        name: "file_manager".to_string(),
        arguments: json!({
            "operation": "list_directory",
            "path": "./examples"
        }).to_string(),
    };
    
    match test_call.validate_against_definition(&function_def) {
        Ok(_) => println!("✅ Function call validation passed"),
        Err(e) => println!("❌ Function call validation failed: {}", e),
    }

    // Demo 5: Advanced Parameter Handling
    println!("\n🎯 Demo 5: Advanced Parameter Handling");
    println!("{}", "=".repeat(50));
    
    let data_tool = DataAnalysisTool::new();
    let data_function_def = FunctionDefinition::from_tool(&data_tool);
    
    println!("Data Analysis Tool Schema:");
    println!("{}", serde_json::to_string_pretty(&data_function_def)?);
    
    // Test parameter extraction
    let complex_call = FunctionCall {
        id: Some("complex_call".to_string()),
        name: "data_analysis".to_string(),
        arguments: json!({
            "data": [10.5, 20.3, 15.7, 25.1, 18.9],
            "analysis_type": "statistics"
        }).to_string(),
    };
    
    let data: Vec<f64> = complex_call.get_parameter("data")?;
    let analysis_type: String = complex_call.get_parameter("analysis_type")?;
    
    println!("Extracted parameters:");
    println!("  Data: {:?}", data);
    println!("  Analysis Type: {}", analysis_type);

    println!("\n🎉 Advanced Tools Demo completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_manager_tool() {
        let tool = FileManagerTool::new(vec!["./test".to_string()]);
        let schema = tool.schema();
        
        assert_eq!(tool.id(), "file_manager");
        assert!(!tool.description().is_empty());
        assert!(!schema.parameters.is_empty());
    }

    #[tokio::test]
    async fn test_data_analysis_tool() {
        let tool = DataAnalysisTool::new();
        let schema = tool.schema();
        
        assert_eq!(tool.id(), "data_analysis");
        assert!(!tool.description().is_empty());
        assert_eq!(schema.parameters.len(), 2);
    }

    #[tokio::test]
    async fn test_function_call_parameter_extraction() {
        let call = FunctionCall {
            id: Some("test".to_string()),
            name: "test_function".to_string(),
            arguments: json!({
                "string_param": "test_value",
                "number_param": 42,
                "array_param": [1, 2, 3]
            }).to_string(),
        };

        let string_val: String = call.get_parameter("string_param").unwrap();
        let number_val: i32 = call.get_parameter("number_param").unwrap();
        let array_val: Vec<i32> = call.get_parameter("array_param").unwrap();

        assert_eq!(string_val, "test_value");
        assert_eq!(number_val, 42);
        assert_eq!(array_val, vec![1, 2, 3]);
    }
}
