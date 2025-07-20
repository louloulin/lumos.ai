//! # 自定义工具示例
//!
//! 本示例展示如何创建和使用自定义工具：
//! - 创建函数式工具
//! - 工具参数验证
//! - 错误处理
//!
//! 运行方式:
//! ```bash
//! cargo run --example custom_tools
//! ```

use lumosai_core::prelude::*;
use lumosai_core::llm::MockLlmProvider;
use lumosai_core::agent::AgentTrait;
use lumosai_core::tool::{Tool, ToolSchema, ParameterSchema, FunctionTool};
use std::sync::Arc;
use anyhow::Result;
use serde_json::{json, Value};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 启动自定义工具示例");
    println!("=========================");
    println!("🔧 自定义工具创建和使用示例");
    println!("=========================\n");

    // 创建LLM提供商
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我将使用自定义计算器工具".to_string(),
        "计算结果已完成".to_string(),
        "我将使用文件处理工具".to_string(),
        "文件处理完成".to_string(),
    ]));

    // 1. 创建函数式工具
    println!("1️⃣ 创建函数式工具");
    println!("------------------");

    let calc_schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "expression".to_string(),
            description: "要计算的数学表达式".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    let calculator_tool = FunctionTool::new(
        "advanced_calculator",
        "高级计算器工具",
        calc_schema,
        |params: Value| {
            let expression = params["expression"].as_str().unwrap_or("0");
            // 简单的计算逻辑
            let result = if expression.contains("+") {
                let parts: Vec<&str> = expression.split("+").collect();
                if parts.len() == 2 {
                    let a: f64 = parts[0].trim().parse().unwrap_or(0.0);
                    let b: f64 = parts[1].trim().parse().unwrap_or(0.0);
                    a + b
                } else {
                    0.0
                }
            } else {
                expression.parse().unwrap_or(0.0)
            };

            Ok(json!({
                "result": result,
                "expression": expression
            }))
        }
    );

    info!("✅ 创建了高级计算器工具");

    // 2. 创建带Agent的工具测试
    println!("\n2️⃣ 测试自定义工具");
    println!("------------------");

    let tools = vec![
        Box::new(calculator_tool) as Box<dyn Tool>,
        calculator(),
        file_reader(),
    ];

    let agent = quick_agent("tool_tester", "工具测试助手")
        .model(llm.clone())
        .tools(tools)
        .build()?;

    info!("✅ 创建了工具测试Agent，工具数量: {}", agent.get_tools().len());

    // 测试工具
    let response = agent.generate_simple("请计算 123 + 456").await?;
    println!("🤖 Agent响应: {}", response);

    println!("\n✅ 自定义工具示例完成!");
    println!("\n📚 相关资源:");
    println!("   - examples/01_getting_started/basic_tools.rs - 基础工具使用");
    println!("   - docs/tools/custom-tools.md - 自定义工具开发指南");

    Ok(())
}