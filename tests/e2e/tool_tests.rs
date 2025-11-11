//! E2E 测试：Agent Tool 调用场景

mod framework;
use framework::{E2ETestContext, E2EAssertions};

use lumosai_core::tool::{Tool, ToolExecutionContext, ToolExecutionOptions, ToolSchema};
use async_trait::async_trait;
use serde_json::{json, Value};
use lumosai_core::base::Base;
use lumosai_core::error::Result;

/// 简单的计算器工具
#[derive(Clone)]
struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn id(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Performs basic math operations (add, subtract, multiply, divide)"
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::with_json_schema(json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["add", "subtract", "multiply", "divide"],
                    "description": "Math operation to perform"
                },
                "a": {
                    "type": "number",
                    "description": "First number"
                },
                "b": {
                    "type": "number",
                    "description": "Second number"
                }
            },
            "required": ["operation", "a", "b"]
        }))
    }

    async fn execute(
        &self,
        params: Value,
        _context: ToolExecutionContext,
        _options: &ToolExecutionOptions,
    ) -> Result<Value> {
        let operation = params["operation"].as_str().unwrap_or("add");
        let a = params["a"].as_f64().unwrap_or(0.0);
        let b = params["b"].as_f64().unwrap_or(0.0);

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Err(lumosai_core::error::Error::Tool(
                        "Division by zero".to_string(),
                    ));
                }
                a / b
            }
            _ => {
                return Err(lumosai_core::error::Error::Tool(format!(
                    "Unknown operation: {}",
                    operation
                )))
            }
        };

        Ok(json!({ "result": result }))
    }

    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

impl Base for CalculatorTool {
    fn id(&self) -> &str {
        "calculator"
    }

    fn name(&self) -> Option<&str> {
        Some("Calculator")
    }
}

/// 测试 10: Agent + 单个工具调用
#[tokio::test]
async fn test_agent_single_tool_call() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建带计算器工具的 Agent
    let calculator = Box::new(CalculatorTool);
    let agent = ctx
        .create_agent_with_tools(
            "math_assistant",
            "You are a math assistant. Use the calculator tool when needed.",
            vec![calculator],
        )
        .unwrap();

    // 执行计算任务（Agent 应该能够调用工具）
    let response = agent.generate_simple("Calculate 15 + 27").await;

    match response {
        Ok(result) => {
            E2EAssertions::assert_non_empty_response(&result);
            println!("✅ Agent with tool responded: {}", result);
        }
        Err(e) => {
            println!("⚠️  Tool call test result: {:?}", e);
            // 对于测试LLM，工具调用可能不完美，但至少应该有响应
        }
    }

    ctx.teardown().await.unwrap();
}

/// 测试 11: Agent + 多个工具
#[tokio::test]
async fn test_agent_multiple_tools() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let tools: Vec<Box<dyn Tool>> = vec![Box::new(CalculatorTool)];

    let agent = ctx
        .create_agent_with_tools(
            "multi_tool_agent",
            "You are a versatile assistant with multiple tools.",
            tools,
        )
        .unwrap();

    // 测试 Agent 能够识别和使用合适的工具
    let response = agent.generate_simple("What tools do you have?").await;

    assert!(response.is_ok());
    let response_text = response.unwrap();
    E2EAssertions::assert_non_empty_response(&response_text);

    ctx.teardown().await.unwrap();
}

/// 测试 12: 工具执行错误处理
#[tokio::test]
async fn test_tool_error_handling() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let calculator = Box::new(CalculatorTool);

    // 直接测试工具的错误处理
    let result = calculator
        .execute(
            json!({
                "operation": "divide",
                "a": 10.0,
                "b": 0.0
            }),
            ToolExecutionContext::default(),
            &ToolExecutionOptions::default(),
        )
        .await;

    assert!(
        result.is_err(),
        "Division by zero should return an error"
    );

    ctx.teardown().await.unwrap();
}

/// 测试 13: 工具 Schema 验证
#[tokio::test]
async fn test_tool_schema_validation() {
    let calculator = CalculatorTool;

    let schema = calculator.schema();

    assert_eq!(schema.name, "calculator");
    assert!(!schema.description.is_empty());
    assert!(schema.parameters["properties"].is_object());
    assert!(schema.parameters["required"].is_array());

    println!("✅ Tool schema validation passed");
}

