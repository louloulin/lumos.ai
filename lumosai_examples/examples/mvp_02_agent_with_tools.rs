//! MVP 示例 2: 带工具的 Agent
//!
//! 这个示例展示如何使用 #[tool] 宏为 Agent 创建和添加工具。
//!
//! 运行方式:
//! ```bash
//! cargo run --package lumosai_examples --example mvp_02_agent_with_tools
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::tool::ToolExecutionContext;
use lumos_macro::tool;
use serde_json::json;

// Re-export lumosai_core modules so macros can use crate:: paths in examples
mod tool {
    pub use lumosai_core::tool::*;
}
mod error {
    pub use lumosai_core::error::*;
}
mod base {
    pub use lumosai_core::base::*;
}
mod compat {
    pub use lumosai_core::compat::*;
}
mod logger {
    pub use lumosai_core::logger::*;
}
mod telemetry {
    pub use lumosai_core::telemetry::*;
}

// 使用 #[tool] 宏创建计算器工具
#[tool(name = "calculator", description = "执行基本的数学运算")]
fn calculator(
    #[parameter(
        name = "operation",
        description = "数学运算类型 (add, subtract, multiply, divide)",
        r#type = "string",
        required = true
    )]
    operation: String,
    #[parameter(
        name = "a",
        description = "第一个数字",
        r#type = "number",
        required = true
    )]
    a: f64,
    #[parameter(
        name = "b",
        description = "第二个数字",
        r#type = "number",
        required = true
    )]
    b: f64,
) -> lumosai_core::Result<serde_json::Value> {
    let result = match operation.as_str() {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Err(lumosai_core::Error::InvalidInput(
                    "除数不能为零".to_string(),
                ));
            }
            a / b
        }
        _ => {
            return Err(lumosai_core::Error::InvalidInput(format!(
                "不支持的运算类型: {}",
                operation
            )))
        }
    };

    Ok(json!({
        "result": result,
        "operation": operation
    }))
}

// 使用 #[tool] 宏创建文本处理工具
#[tool(name = "text_processor", description = "处理文本字符串")]
fn text_processor(
    #[parameter(
        name = "text",
        description = "输入文本",
        r#type = "string",
        required = true
    )]
    text: String,
    #[parameter(
        name = "operation",
        description = "操作类型 (uppercase, lowercase, reverse, length)",
        r#type = "string",
        required = true
    )]
    operation: String,
) -> lumosai_core::Result<serde_json::Value> {
    let result = match operation.as_str() {
        "uppercase" => json!({"result": text.to_uppercase()}),
        "lowercase" => json!({"result": text.to_lowercase()}),
        "reverse" => json!({"result": text.chars().rev().collect::<String>()}),
        "length" => json!({"result": text.len()}),
        _ => {
            return Err(lumosai_core::Error::InvalidInput(format!(
                "不支持的操作类型: {}",
                operation
            )))
        }
    };

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 2: 带工具的 Agent\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 使用 #[tool] 宏创建的工具
    println!("\n📝 步骤 1: 使用 #[tool] 宏创建工具");
    // 宏会生成 calculator_tool() 和 text_processor_tool() 工厂函数
    let calculator = calculator_tool();
    let text_processor = text_processor_tool();
    println!("✅ 创建了 2 个工具:");
    println!("   - {}: {}", calculator.id(), calculator.description());
    println!("   - {}: {}", text_processor.id(), text_processor.description());

    // 步骤 2: 测试工具调用
    println!("\n📝 步骤 2: 测试工具调用");
    println!("{}", "━".repeat(50));

    // 测试计算器工具
    println!("\n🧮 测试计算器工具:");
    let calc_params = json!({
        "operation": "add",
        "a": 25.5,
        "b": 14.3
    });
    
    let context = ToolExecutionContext::default();
    let calc_result = calculator.execute(calc_params, context.clone(), &Default::default()).await?;
    println!("   输入: 25.5 + 14.3");
    println!("   结果: {}", calc_result);

    // 测试文本处理工具
    println!("\n📝 测试文本处理工具:");
    let text_params = json!({
        "text": "Hello World",
        "operation": "uppercase"
    });
    
    let text_result = text_processor.execute(text_params, context, &Default::default()).await?;
    println!("   输入: 'Hello World' -> uppercase");
    println!("   结果: {}", text_result);

    // 步骤 3: 创建 Agent
    println!("\n📝 步骤 3: 创建带工具的 Agent");
    let llm = create_test_zhipu_provider_arc();
    let agent = AgentBuilder::new()
        .name("tool_agent")
        .instructions("You are a helpful assistant with access to tools. You can perform calculations and text processing.")
        .model(llm)
        .build()?;
    println!("✅ Agent '{}' 创建成功", agent.name().unwrap_or("unknown"));

    // 步骤 4: Agent 对话示例
    println!("\n📝 步骤 4: Agent 对话示例");
    println!("{}", "━".repeat(50));

    let questions = vec![
        "Hi! What can you help me with?",
        "Can you explain how tools work?",
    ];

    for (i, question) in questions.iter().enumerate() {
        println!("\n💬 问题 {}: {}", i + 1, question);
        match agent.generate_simple(question).await {
            Ok(response) => {
                println!("🤖 Agent: {}", response);
            }
            Err(e) => {
                eprintln!("❌ 错误: {}", e);
            }
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("✅ 工具系统示例完成！");
    println!("\n💡 关键概念:");
    println!("   - 使用 #[tool] 宏定义工具");
    println!("   - 使用 #[parameter] 属性定义参数");
    println!("   - 工具是扩展 Agent 能力的方式");
    println!("   - Agent 可以调用工具来完成复杂任务");
    println!("\n🔧 #[tool] 宏的优势:");
    println!("   ✅ 自动生成 Tool trait 实现");
    println!("   ✅ 自动参数验证");
    println!("   ✅ 类型安全");
    println!("   ✅ 减少样板代码");
    println!("   ✅ 清晰的参数描述");
    println!("\n📦 工具参数属性:");
    println!("   - name: 参数名称");
    println!("   - description: 参数描述");
    println!("   - r#type: 参数类型（string, number, integer, boolean）");
    println!("   - required: 是否必需");
    println!("\n下一步: 查看 mvp_03_multi_agent.rs 学习多 Agent 协作");

    Ok(())
}
