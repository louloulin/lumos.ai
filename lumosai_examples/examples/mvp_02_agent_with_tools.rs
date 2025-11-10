//! MVP 示例 2: 带工具的 Agent
//!
//! 这个示例展示如何为 Agent 添加工具，扩展其能力。
//!
//! 运行方式:
//! ```bash
//! cargo run --example mvp_02_agent_with_tools
//! ```

use lumosai_core::agent::{create_basic_agent, Agent};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::llm::LlmProvider;
use lumosai_core::tool::{Tool, ToolBuilder, ToolExecutionContext, ToolExecutionOptions};
use serde_json::json;
use std::sync::Arc;

/// 创建一个计算器工具
fn create_calculator_tool() -> Arc<dyn Tool> {
    ToolBuilder::new("calculator")
        .description("Performs basic arithmetic operations (add, subtract, multiply, divide)")
        .parameter("a", "number", "First number", true)
        .parameter("b", "number", "Second number", true)
        .parameter("operation", "string", "Operation: add, subtract, multiply, divide", true)
        .handler(|args, _ctx| {
            Box::pin(async move {
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
                
                Ok(json!({
                    "result": result,
                    "operation": format!("{} {} {} = {}", a, op, b, result)
                }))
            })
        })
        .build()
}

/// 创建一个字符串工具
fn create_string_tool() -> Arc<dyn Tool> {
    ToolBuilder::new("string_utils")
        .description("String utility functions (uppercase, lowercase, reverse, length)")
        .parameter("text", "string", "Input text", true)
        .parameter("operation", "string", "Operation: uppercase, lowercase, reverse, length", true)
        .handler(|args, _ctx| {
            Box::pin(async move {
                let text = args["text"].as_str().ok_or("Invalid text")?;
                let op = args["operation"].as_str().ok_or("Invalid operation")?;
                
                let result = match op {
                    "uppercase" => json!({ "result": text.to_uppercase() }),
                    "lowercase" => json!({ "result": text.to_lowercase() }),
                    "reverse" => json!({ "result": text.chars().rev().collect::<String>() }),
                    "length" => json!({ "result": text.len() }),
                    _ => return Err(format!("Unknown operation: {}", op).into()),
                };
                
                Ok(result)
            })
        })
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 2: 带工具的 Agent\n");
    println!("=".repeat(50));
    
    // 步骤 1: 创建工具
    println!("\n📝 步骤 1: 创建工具");
    let calculator = create_calculator_tool();
    let string_utils = create_string_tool();
    println!("✅ 创建了 2 个工具:");
    println!("   - calculator: 基本算术运算");
    println!("   - string_utils: 字符串处理");
    
    // 步骤 2: 创建 LLM 提供商
    println!("\n📝 步骤 2: 创建 LLM 提供商");
    let llm: Arc<dyn LlmProvider> = create_test_zhipu_provider_arc();
    println!("✅ LLM 提供商创建成功");
    
    // 步骤 3: 创建 Agent 并注册工具
    println!("\n📝 步骤 3: 创建 Agent 并注册工具");
    let mut agent = create_basic_agent(
        "tool_agent",
        "You are a helpful assistant with access to tools. Use the calculator for math and string_utils for text processing.",
        llm,
    );
    
    // 注册工具
    agent.register_tool(calculator)?;
    agent.register_tool(string_utils)?;
    println!("✅ Agent '{}' 创建成功，已注册 2 个工具", agent.name());
    
    // 步骤 4: 测试工具使用
    println!("\n📝 步骤 4: 测试工具使用");
    println!("━".repeat(50));
    
    // 测试计算器
    println!("\n💬 测试 1: 使用计算器");
    println!("问题: What is 123 + 456?");
    
    let ctx = ToolExecutionContext::default();
    let options = ToolExecutionOptions::default();
    
    let calc_result = agent
        .get_tool("calculator")
        .ok_or("Calculator tool not found")?
        .execute(
            json!({
                "a": 123,
                "b": 456,
                "operation": "add"
            }),
            &ctx,
            &options,
        )
        .await?;
    
    println!("🤖 计算结果: {}", calc_result);
    
    // 测试字符串工具
    println!("\n💬 测试 2: 使用字符串工具");
    println!("问题: Convert 'Hello World' to uppercase");
    
    let string_result = agent
        .get_tool("string_utils")
        .ok_or("String utils tool not found")?
        .execute(
            json!({
                "text": "Hello World",
                "operation": "uppercase"
            }),
            &ctx,
            &options,
        )
        .await?;
    
    println!("🤖 转换结果: {}", string_result);
    
    // 测试更多操作
    println!("\n💬 测试 3: 更多操作");
    
    let operations = vec![
        ("multiply", json!({"a": 12, "b": 34, "operation": "multiply"})),
        ("reverse", json!({"text": "LumosAI", "operation": "reverse"})),
        ("divide", json!({"a": 100, "b": 5, "operation": "divide"})),
    ];
    
    for (name, args) in operations {
        let tool_name = if name.contains("multiply") || name.contains("divide") {
            "calculator"
        } else {
            "string_utils"
        };
        
        match agent.get_tool(tool_name) {
            Some(tool) => {
                match tool.execute(args.clone(), &ctx, &options).await {
                    Ok(result) => println!("✅ {}: {}", name, result),
                    Err(e) => eprintln!("❌ {}: {}", name, e),
                }
            }
            None => eprintln!("❌ Tool '{}' not found", tool_name),
        }
    }
    
    println!("\n" + &"=".repeat(50));
    println!("✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - 工具可以扩展 Agent 的能力");
    println!("   - 使用 ToolBuilder 创建自定义工具");
    println!("   - Agent 可以注册多个工具");
    println!("   - 下一步: 查看 mvp_03_multi_agent.rs 学习多 Agent 协作");
    
    Ok(())
}

