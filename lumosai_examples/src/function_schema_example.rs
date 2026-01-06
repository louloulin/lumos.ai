//! Example demonstrating the new FunctionSchema derive macro
//!
//! This example shows how to use the derive macro to automatically generate
//! OpenAI function calling schemas from Rust structs.

use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::{
    agent::{Agent, AgentConfig, BasicAgent},
    llm::{LlmOptions, Message, Role},
    Result,
};
// use lumosai_derive::FunctionSchema; // Temporarily disabled - package excluded
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Calculator parameters (simplified - macros temporarily disabled)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatorParams {
    /// The mathematical expression to evaluate (e.g., "2 + 3 * 4")
    pub expression: String,
    /// Number of decimal places for precision (optional, defaults to 2)
    pub precision: Option<u32>,
    /// Whether to show step-by-step calculation (optional)
    pub show_steps: Option<bool>,
}

/// Weather query parameters (simplified - macros temporarily disabled)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherParams {
    /// The city name to get weather for
    pub city: String,
    /// Country code (optional, ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// Temperature unit preference
    pub unit: Option<TemperatureUnit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemperatureUnit {
    Celsius,
    Fahrenheit,
    Kelvin,
}

/// Search parameters for web search functionality (simplified - macros temporarily disabled)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    /// The search query
    pub query: String,
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Language preference for results
    pub language: Option<String>,
    /// Whether to include images in results
    pub include_images: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Function Schema Derive Macro Example");
    println!("========================================\n");

    // Test the automatic schema generation
    test_schema_generation().await?;

    // Test with a real agent
    test_agent_with_function_calling().await?;

    Ok(())
}

async fn test_schema_generation() -> Result<()> {
    println!("📋 Testing schema structures (macros temporarily disabled)...\n");

    // Test Calculator structure
    let calc_params = CalculatorParams {
        expression: "2 + 3 * 4".to_string(),
        precision: Some(2),
        show_steps: Some(true),
    };
    println!("Calculator Parameters:");
    println!("{:#?}\n", calc_params);

    // Test Weather structure
    let weather_params = WeatherParams {
        city: "San Francisco".to_string(),
        country: Some("US".to_string()),
        unit: Some(TemperatureUnit::Celsius),
    };
    println!("Weather Parameters:");
    println!("{:#?}\n", weather_params);

    // Test Search structure
    let search_params = SearchParams {
        query: "Rust programming".to_string(),
        limit: Some(10),
        language: Some("en".to_string()),
        include_images: Some(false),
    };
    println!("Search Parameters:");
    println!("{:#?}\n", search_params);

    // Test validation (simplified)
    test_parameter_validation().await?;

    Ok(())
}

async fn test_parameter_validation() -> Result<()> {
    println!("✅ Testing parameter validation (simplified)...\n");

    // Valid parameters - test JSON deserialization
    let valid_calc_json = serde_json::json!({
        "expression": "2 + 3 * 4",
        "precision": 2,
        "show_steps": true
    });

    match serde_json::from_value::<CalculatorParams>(valid_calc_json) {
        Ok(_) => println!("✓ Valid calculator arguments passed JSON deserialization"),
        Err(e) => println!(
            "✗ Valid calculator arguments failed JSON deserialization: {}",
            e
        ),
    }

    // Invalid parameters (missing required field)
    let invalid_args = serde_json::json!({"precision": 2});

    match serde_json::from_value::<CalculatorParams>(invalid_args) {
        Ok(_) => println!("✗ Invalid arguments incorrectly passed validation"),
        Err(_) => println!("✓ Invalid arguments correctly failed validation"),
    }

    println!();
    Ok(())
}

async fn test_agent_with_function_calling() -> Result<()> {
    println!("🤖 Testing agent with function calling...\n");

    // Create a mock LLM provider that returns function calls
    let mock_llm = Arc::new(MockLlmWithFunctionCalling::new());

    // Create agent configuration
    let config = AgentConfig {
        name: "function_test_agent".to_string(),
        instructions: "You are a helpful assistant that can perform calculations, get weather, and search the web.".to_string(),
        memory_config: None,
        working_memory: None,
        enable_function_calling: Some(true),
        ..Default::default()
    };

    // Create the agent
    let agent = BasicAgent::new(config, mock_llm);

    // Note: In a real implementation, you would add actual tools here
    // For this example, we're just testing the schema generation

    // Test message
    let messages = vec![Message {
        role: Role::User,
        content: "Calculate 15 * 23 + 7".to_string(),
        name: None,
        metadata: None,
    }];

    let options = AgentGenerateOptions::default();

    println!("Sending message to agent: 'Calculate 15 * 23 + 7'");

    match agent.generate(&messages, &options).await {
        Ok(result) => {
            println!("✓ Agent response: {}", result.response);
            println!("✓ Steps completed: {}", result.steps.len());
        }
        Err(e) => {
            println!("✗ Agent failed: {}", e);
        }
    }

    Ok(())
}

/// Mock LLM provider that simulates function calling responses
pub struct MockLlmWithFunctionCalling {
    responses: Vec<String>,
    index: std::sync::atomic::AtomicUsize,
}

impl MockLlmWithFunctionCalling {
    pub fn new() -> Self {
        Self {
            responses: vec![
                "I'll calculate 15 * 23 + 7 for you.".to_string(),
                "The result is 352.".to_string(),
            ],
            index: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[async_trait::async_trait]
impl lumosai_core::llm::LlmProvider for MockLlmWithFunctionCalling {
    fn name(&self) -> &str {
        "MockLlmWithFunctionCalling"
    }

    async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
        let index =
            self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % self.responses.len();
        Ok(self.responses[index].clone())
    }

    async fn generate_with_messages(
        &self,
        _messages: &[Message],
        _options: &LlmOptions,
    ) -> Result<String> {
        let index =
            self.index.fetch_add(1, std::sync::atomic::Ordering::SeqCst) % self.responses.len();
        Ok(self.responses[index].clone())
    }

    async fn generate_stream<'a>(
        &'a self,
        _prompt: &'a str,
        _options: &'a LlmOptions,
    ) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
        unimplemented!("Streaming not implemented for mock provider")
    }

    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        unimplemented!("Embeddings not implemented for mock provider")
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    async fn generate_with_functions(
        &self,
        _messages: &[Message],
        _functions: &[lumosai_core::llm::function_calling::FunctionDefinition],
        _tool_choice: &lumosai_core::llm::function_calling::ToolChoice,
        _options: &LlmOptions,
    ) -> Result<lumosai_core::llm::provider::FunctionCallingResponse> {
        // Simulate a function call response
        use lumosai_core::llm::{
            function_calling::FunctionCall, provider::FunctionCallingResponse,
        };

        let function_call = FunctionCall {
            id: Some("call_123".to_string()),
            name: "calculate".to_string(),
            arguments: serde_json::json!({
                "expression": "15 * 23 + 7",
                "precision": 2
            })
            .to_string(),
        };

        Ok(FunctionCallingResponse {
            content: Some("I'll calculate that for you.".to_string()),
            function_calls: vec![function_call],
            finish_reason: "function_call".to_string(),
        })
    }
}
