//! DSL 宏演示
//!
//! 演示如何使用 DSL 宏快速创建 Agent、Tool 和 Workflow：
//! - `agent!` - 快速创建 Agent
//! - `#[tool]` - 定义工具
//! - `workflow!` - 定义工作流

use lumos_macro::{agent, tool};
use lumosai_core::agent::Agent;
use lumosai_core::llm::openai::OpenAIProvider;
use lumosai_core::tool::Tool;
use std::sync::Arc;

// ========================================
// 1. 使用 #[tool] 宏定义工具
// ========================================

/// 计算器工具
#[tool(
    name = "calculator",
    description = "Perform basic arithmetic operations"
)]
async fn calculator(
    /// The first number
    a: f64,
    /// The second number
    b: f64,
    /// The operation to perform (add, subtract, multiply, divide)
    operation: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err("Division by zero".into());
            }
            a / b
        }
        _ => return Err(format!("Unknown operation: {}", operation).into()),
    };

    Ok(format!("{} {} {} = {}", a, operation, b, result))
}

/// 搜索工具
#[tool(name = "web_search", description = "Search the web for information")]
async fn web_search(
    /// The search query
    query: String,
) -> Result<String, Box<dyn std::error::Error>> {
    // 模拟搜索
    Ok(format!("Search results for: {}", query))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DSL 宏演示\n");

    // ========================================
    // 2. 使用 agent! 宏创建 Agent（基础用法）
    // ========================================
    println!("📊 1. 基础 Agent 创建");

    let basic_agent = agent! {
        name: "basic_assistant",
        instructions: "You are a helpful assistant",
        model: "gpt-4"
    };

    println!("   ✅ 创建了 Agent: {}\n", basic_agent.name());

    // ========================================
    // 3. 使用 agent! 宏创建带工具的 Agent
    // ========================================
    println!("📊 2. 带工具的 Agent 创建");

    let tool_agent = agent! {
        name: "tool_assistant",
        instructions: "You are an assistant with tools",
        model: "gpt-4",
        tools: [calculator, web_search]
    };

    println!("   ✅ 创建了带工具的 Agent: {}\n", tool_agent.name());

    // ========================================
    // 4. 使用 agent! 宏创建带自定义提供者的 Agent
    // ========================================
    println!("📊 3. 自定义提供者 Agent 创建");

    let provider = OpenAIProvider::from_env()?;
    let custom_agent = agent! {
        name: "custom_assistant",
        instructions: "You are a custom assistant",
        provider: provider,
        tools: [calculator]
    };

    println!("   ✅ 创建了自定义 Agent: {}\n", custom_agent.name());

    // ========================================
    // 5. 测试工具
    // ========================================
    println!("📊 4. 测试工具");

    let calc_tool = calculator();
    println!("   工具名称: {}", calc_tool.id());
    println!("   工具描述: {}", calc_tool.description());

    // 测试工具执行
    let tool_input = serde_json::json!({
        "a": 10.0,
        "b": 5.0,
        "operation": "add"
    });

    let tool_context = lumosai_core::tool::ToolContext {
        thread_id: Some("test-thread".to_string()),
        user_id: Some("test-user".to_string()),
        metadata: Default::default(),
    };

    let result = calc_tool.execute(&tool_input, &tool_context).await?;
    println!("   ✅ 工具执行结果: {}\n", result);

    // ========================================
    // 6. 测试 Agent 生成
    // ========================================
    println!("📊 5. 测试 Agent 生成");

    let messages = vec![lumosai_core::llm::Message {
        role: lumosai_core::llm::Role::User,
        content: "Hello, how are you?".to_string(),
        metadata: None,
        name: None,
    }];

    println!("   发送消息: Hello, how are you?");
    let response = basic_agent.generate(&messages, &Default::default()).await?;
    println!("   ✅ Agent 响应: {}\n", response.response);

    println!("✅ 所有宏演示完成！");

    Ok(())
}
