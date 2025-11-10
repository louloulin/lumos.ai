//! MVP 示例 2: 带工具的 Agent
//!
//! 这个示例展示如何使用 AgentBuilder 和 ToolBuilder 为 Agent 添加工具。
//!
//! 运行方式:
//! ```bash
//! cargo run --example mvp_02_agent_with_tools
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::tool::{Tool, ToolBuilder};
use serde_json::json;

/// 创建一个计算器工具
fn create_calculator_tool() -> Box<dyn Tool> {
    Box::new(
        ToolBuilder::new()
            .name("calculator")
            .description("Performs basic arithmetic operations")
            .parameter("a", "number", "First number", true)
            .parameter("b", "number", "Second number", true)
            .parameter("operation", "string", "Operation: add, subtract, multiply, divide", true)
            .handler(|args| {
                let a = args["a"].as_f64().ok_or("Invalid number 'a'")?;
                let b = args["b"].as_f64().ok_or("Invalid number 'b'")?;
                let op = args["operation"].as_str().ok_or("Invalid operation")?;

                let result = match op {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0.0 {
                            return Err("Cannot divide by zero".into());
                        }
                        a / b
                    }
                    _ => return Err(format!("Unknown operation: {}", op).into()),
                };

                Ok(json!({"result": result}))
            })
            .build()
            .expect("Failed to build calculator tool")
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 2: 带工具的 Agent\n");
    println!("{}", "=".repeat(50));
    
    let calculator = create_calculator_tool();
    let llm = create_test_zhipu_provider_arc();
    
    let agent = AgentBuilder::new()
        .name("tool_agent")
        .instructions("You are a helpful assistant.")
        .model(llm)
        .tool(calculator)
        .build()?;
    
    println!("✅ Agent '{}' 创建成功", agent.name().unwrap_or("unknown"));
    
    let response = agent.generate_simple("What is 123 + 456?").await?;
    println!("🤖 Agent: {}", response);
    
    println!("\n✅ 示例完成！");
    Ok(())
}
