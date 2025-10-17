//! Demonstration of Phase 1 OpenAI Function Calling enhancements
//!
//! This example shows how the modernized function calling system works,
//! including the distinction between function calling mode and legacy regex mode.

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use lumosai_core::agent::executor::BasicAgent;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::agent::{Agent, AgentConfig};
use lumosai_core::base::{Base, BaseComponent, ComponentConfig};
use lumosai_core::llm::function_calling::{FunctionCall, FunctionDefinition, ToolChoice};
use lumosai_core::llm::provider::FunctionCallingResponse;
use lumosai_core::llm::{LlmOptions, LlmProvider, Message, Role};
use lumosai_core::compat::{Logger, TelemetrySink, Component, Event};
use lumosai_core::tool::{
    ParameterSchema, Tool, ToolExecutionContext, ToolExecutionOptions, ToolSchema,
};
use lumosai_core::{Error, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// Mock LLM provider with function calling support for testing
pub struct MockLlmWithFunctionCalling {
    pub supports_function_calling: bool,
}

impl MockLlmWithFunctionCalling {
    pub fn new(supports_function_calling: bool) -> Self {
        Self {
            supports_function_calling,
        }
    }
}

#[async_trait]
impl LlmProvider for MockLlmWithFunctionCalling {
    fn name(&self) -> &str {
        "MockLlmWithFunctionCalling"
    }

    async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
        if self.supports_function_calling {
            Ok("I'll use function calling to help you.".to_string())
        } else {
            Ok(format!("思考: I need to calculate something\n工具: calculator\n参数: {{\"expression\": \"2+2\"}}\n结果: 4"))
        }
    }

    async fn generate_with_messages(
        &self,
        messages: &[Message],
        options: &LlmOptions,
    ) -> Result<String> {
        self.generate(&messages.last().unwrap().content, options)
            .await
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        let response = self.generate(prompt, options).await?;
        let stream = stream::once(async move { Ok(response) });
        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.1, 0.2, 0.3])
    }

    fn supports_function_calling(&self) -> bool {
        self.supports_function_calling
    }

    async fn generate_with_functions(
        &self,
        _messages: &[Message],
        _functions: &[FunctionDefinition],
        _tool_choice: &ToolChoice,
        _options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        if self.supports_function_calling {
            // Simulate a function call response
            let function_call = FunctionCall {
                id: Some("call_123".to_string()),
                name: "calculator".to_string(),
                arguments: json!({
                    "expression": "2+2"
                })
                .to_string(),
            };

            Ok(FunctionCallingResponse {
                content: Some("I'll calculate that for you.".to_string()),
                function_calls: vec![function_call],
                finish_reason: "function_call".to_string(),
            })
        } else {
            // Fall back to regular generation
            let response = self.generate_with_messages(_messages, _options).await?;
            Ok(FunctionCallingResponse {
                content: Some(response),
                function_calls: vec![],
                finish_reason: "stop".to_string(),
            })
        }
    }
}

/// Simple calculator tool for testing
#[derive(Clone)]
pub struct CalculatorTool {
    base: BaseComponent,
    id: String,
}

// Manual Debug implementation since BaseComponent doesn't derive Debug
impl std::fmt::Debug for CalculatorTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CalculatorTool")
            .field("id", &self.id)
            .field("name", &self.base.name())
            .field("component", &self.base.component())
            .finish()
    }
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            base: BaseComponent::new(ComponentConfig::default()),
            id: "calculator".to_string(),
        }
    }
}

impl Base for CalculatorTool {
    fn name(&self) -> Option<&str> {
        self.base.name()
    }

    fn component(&self) -> Component {
        self.base.component()
    }

    fn logger(&self) -> Arc<dyn Logger> {
        self.base.logger()
    }

    fn set_logger(&mut self, logger: Arc<dyn Logger>) {
        self.base.set_logger(logger);
    }

    fn telemetry(&self) -> Option<Arc<dyn TelemetrySink>> {
        self.base.telemetry()
    }

    fn set_telemetry(&mut self, telemetry: Arc<dyn TelemetrySink>) {
        self.base.set_telemetry(telemetry);
    }
}

#[async_trait]
impl Tool for CalculatorTool {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        "A simple calculator that can evaluate mathematical expressions"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(vec![ParameterSchema {
            name: "expression".to_string(),
            description: "Mathematical expression to evaluate".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        }])
    }

    async fn execute(
        &self,
        params: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions,
    ) -> Result<Value> {
        let expression = params
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::InvalidInput("Missing expression parameter".to_string()))?;

        // Simple calculator for demo (just handle basic additions)
        let result = if expression == "2+2" {
            4
        } else {
            42 // Default result for demo
        };

        Ok(json!(result))
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

/// Mock logger for testing
pub struct MockLogger;

impl Logger for MockLogger {
    fn log(&self, level: lumosai_core::compat::LogLevel, message: &str) {
        println!("[{:?}] {}", level, message);
    }
}

/// Mock telemetry sink for testing
pub struct MockTelemetry;

#[async_trait]
impl TelemetrySink for MockTelemetry {
    async fn send_event(&self, _event: Event) {
        // No-op for testing
    }

    fn record_event(&self, _event: serde_json::Value) {
        // No-op for testing
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== LumosAI Function Calling Enhancement Demo ===\n");

    // Test 1: Agent with function calling enabled LLM
    println!("🔧 Test 1: Agent with Function Calling Support");
    println!("============================================");

    let llm_with_fc = Arc::new(MockLlmWithFunctionCalling::new(true));
    let config_with_fc = AgentConfig {
        name: "test_agent_fc".to_string(),
        instructions: "You are a helpful assistant with calculator capabilities.".to_string(),
        enable_function_calling: Some(true),
        ..Default::default()
    };

    let mut agent_with_fc = BasicAgent::new(config_with_fc, llm_with_fc);

    // Add calculator tool
    agent_with_fc.add_tool(Box::new(CalculatorTool::new()))?;

    let messages_fc = vec![Message {
        role: Role::User,
        content: "Please calculate 2+2".to_string(),
        metadata: None,
        name: None,
    }];

    let options_fc = AgentGenerateOptions::default();
    let result_fc = agent_with_fc.generate(&messages_fc, &options_fc).await?;

    println!("User: Please calculate 2+2");
    println!("Agent (Function Calling): {}", result_fc.response);
    println!("Steps: {}", result_fc.steps.len());
    println!();

    // Test 2: Agent with legacy regex mode LLM
    println!("🔧 Test 2: Agent with Legacy Regex Mode");
    println!("======================================");

    let llm_legacy = Arc::new(MockLlmWithFunctionCalling::new(false));
    let config_legacy = AgentConfig {
        name: "test_agent_legacy".to_string(),
        instructions: "You are a helpful assistant with calculator capabilities.".to_string(),
        enable_function_calling: Some(true), // Still enabled, but LLM doesn't support it
        ..Default::default()
    };

    let mut agent_legacy = BasicAgent::new(config_legacy, llm_legacy);

    // Add calculator tool
    agent_legacy.add_tool(Box::new(CalculatorTool::new()))?;

    let messages_legacy = vec![Message {
        role: Role::User,
        content: "Please calculate 2+2".to_string(),
        metadata: None,
        name: None,
    }];

    let options_legacy = AgentGenerateOptions::default();
    let result_legacy = agent_legacy
        .generate(&messages_legacy, &options_legacy)
        .await?;

    println!("User: Please calculate 2+2");
    println!("Agent (Legacy Regex): {}", result_legacy.response);
    println!("Steps: {}", result_legacy.steps.len());
    println!();

    // Test 3: Agent with function calling disabled
    println!("🔧 Test 3: Agent with Function Calling Disabled");
    println!("==============================================");

    let llm_disabled = Arc::new(MockLlmWithFunctionCalling::new(true));
    let config_disabled = AgentConfig {
        name: "test_agent_disabled".to_string(),
        instructions: "You are a helpful assistant with calculator capabilities.".to_string(),
        enable_function_calling: Some(false), // Explicitly disabled
        ..Default::default()
    };

    let mut agent_disabled = BasicAgent::new(config_disabled, llm_disabled);

    // Add calculator tool
    agent_disabled.add_tool(Box::new(CalculatorTool::new()))?;

    let messages_disabled = vec![Message {
        role: Role::User,
        content: "Please calculate 2+2".to_string(),
        metadata: None,
        name: None,
    }];

    let options_disabled = AgentGenerateOptions::default();
    let result_disabled = agent_disabled
        .generate(&messages_disabled, &options_disabled)
        .await?;

    println!("User: Please calculate 2+2");
    println!(
        "Agent (Function Calling Disabled): {}",
        result_disabled.response
    );
    println!("Steps: {}", result_disabled.steps.len());
    println!();

    println!("✅ All tests completed successfully!");
    println!("\n📋 Summary:");
    println!("- Function calling mode: Cleaner system messages, native OpenAI function calling");
    println!("- Legacy regex mode: Detailed tool descriptions in system message");
    println!("- Disabled mode: Falls back to regex parsing even with capable LLM");

    Ok(())
}
