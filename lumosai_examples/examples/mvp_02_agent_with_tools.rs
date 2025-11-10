//! MVP 示例 2: 带工具的 Agent
//!
//! 这个示例展示如何使用 AgentBuilder 和 tool! 宏为 Agent 添加工具。
//!
//! 运行方式:
//! ```bash
//! cargo run --example mvp_02_agent_with_tools
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::tool::Tool;
use lumos_macro::tool;
use serde_json::json;

// 使用 tool! 宏创建计算器工具
tool! {
    name: "calculator",
    description: "Performs basic arithmetic operations",
    parameters: [
        {
            name: "a",
            description: "First number",
            type: "number",
            required: true
        },
        {
            name: "b",
            description: "Second number",
            type: "number",
            required: true
        },
        {
            name: "operation",
            description: "Operation: add, subtract, multiply, divide",
            type: "string",
            required: true
        }
    ],
    handler: |params| async move {
        let a = params["a"].as_f64().ok_or("Invalid number 'a'")?;
        let b = params["b"].as_f64().ok_or("Invalid number 'b'")?;
        let op = params["operation"].as_str().ok_or("Invalid operation")?;

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
    }
}

// 使用 tool! 宏创建字符串工具
tool! {
    name: "string_utils",
    description: "String utility functions",
    parameters: [
        {
            name: "text",
            description: "Input text",
            type: "string",
            required: true
        },
        {
            name: "operation",
            description: "Operation: uppercase, lowercase, reverse, length",
            type: "string",
            required: true
        }
    ],
    handler: |params| async move {
        let text = params["text"].as_str().ok_or("Invalid text")?;
        let op = params["operation"].as_str().ok_or("Invalid operation")?;

        let result = match op {
            "uppercase" => json!({"result": text.to_uppercase()}),
            "lowercase" => json!({"result": text.to_lowercase()}),
            "reverse" => json!({"result": text.chars().rev().collect::<String>()}),
            "length" => json!({"result": text.len()}),
            _ => return Err(format!("Unknown operation: {}", op).into()),
        };

        Ok(result)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 2: 带工具的 Agent\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 创建工具（使用 tool! 宏）
    println!("\n📝 步骤 1: 创建工具（使用 tool! 宏）");
    let calculator = calculator_tool();
    let string_utils = string_utils_tool();
    println!("✅ 创建了 2 个工具:");
    println!("   - calculator: 基本算术运算");
    println!("   - string_utils: 字符串处理");

    // 步骤 2: 创建 LLM 提供商
    println!("\n📝 步骤 2: 创建 LLM 提供商");
    let llm = create_test_zhipu_provider_arc();
    println!("✅ LLM 提供商创建成功");

    // 步骤 3: 使用 AgentBuilder 创建 Agent 并添加工具
    println!("\n📝 步骤 3: 使用 AgentBuilder 创建 Agent 并添加工具");
    let agent = AgentBuilder::new()
        .name("tool_agent")
        .instructions("You are a helpful assistant with access to tools. Use the calculator for math and string_utils for text processing.")
        .model(llm)
        .tool(calculator)
        .tool(string_utils)
        .max_tool_calls(10)
        .tool_timeout(30)
        .build()?;

    println!("✅ Agent '{}' 创建成功，已注册 2 个工具", agent.name().unwrap_or("unknown"));

    // 步骤 4: 测试 Agent 对话
    println!("\n📝 步骤 4: 测试 Agent 对话");
    println!("{}", "━".repeat(50));

    let questions = vec![
        "What is 123 + 456?",
        "Convert 'Hello World' to uppercase",
        "What is 100 divided by 5?",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("\n💬 问题 {}: {}", i + 1, question);
        match agent.generate_simple(question).await {
            Ok(response) => println!("🤖 Agent: {}", response),
            Err(e) => eprintln!("❌ 错误: {}", e),
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("✅ 示例完成！");
    println!("\n💡 提示:");
    println!("   - 使用 tool! 宏创建工具（声明式语法）");
    println!("   - 宏会自动生成 <tool_name>_tool() 函数");
    println!("   - 使用 AgentBuilder.tool() 添加工具到 Agent");
    println!("   - 可以添加多个工具，Agent 会自动选择合适的工具");
    println!("   - 下一步: 查看 mvp_03_multi_agent.rs 学习多 Agent 协作");

    Ok(())
}
