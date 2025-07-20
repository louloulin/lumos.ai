//! # 工具集成示例
//!
//! 本示例展示如何在LumosAI中集成和使用工具：
//! - 内置工具使用
//! - 多工具组合
//! - 工具状态监控
//! - 智能工具选择
//!
//! 运行方式:
//! ```bash
//! cargo run --example mcp_integration
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

    info!("🚀 启动工具集成示例");
    println!("=========================");
    println!("🔧 工具集成和使用示例");
    println!("=========================\n");

    // 创建LLM提供商
    let llm = Arc::new(MockLlmProvider::new(vec![
        "我已经获取了可用的工具列表".to_string(),
        "正在使用计算器工具进行计算...".to_string(),
        "根据工具的结果，计算完成".to_string(),
        "我已经使用多个工具完成了任务".to_string(),
    ]));

    // 1. 基础工具集成
    println!("1️⃣ 基础工具集成");
    println!("----------------");

    info!("📡 加载内置工具");

    // 创建自定义工具
    let weather_schema = ToolSchema::new(vec![
        ParameterSchema {
            name: "city".to_string(),
            description: "城市名称".to_string(),
            r#type: "string".to_string(),
            required: true,
            properties: None,
            default: None,
        },
    ]);

    let weather_tool = FunctionTool::new(
        "weather_query",
        "天气查询工具",
        weather_schema,
        |params: Value| {
            let city = params["city"].as_str().unwrap_or("北京");
            Ok(json!({
                "city": city,
                "temperature": "25°C",
                "weather": "晴朗",
                "humidity": "60%"
            }))
        }
    );

    // 创建工具集合
    let tools = vec![
        calculator(),
        file_reader(),
        file_writer(),
        Box::new(weather_tool) as Box<dyn Tool>,
    ];

    info!("✅ 加载了 {} 个工具", tools.len());
    for (i, tool) in tools.iter().enumerate() {
        println!("   🔧 工具 {}: {}", i + 1, tool.name().unwrap_or("未知工具"));
    }

    // 2. 创建集成工具的Agent
    println!("\n2️⃣ 工具集成Agent");
    println!("------------------");

    let tool_agent = quick_agent("tool_assistant", "工具集成助手")
        .model(llm.clone())
        .tools(tools)
        .build()?;

    info!("✅ 工具Agent创建成功，工具数量: {}", tool_agent.get_tools().len());

    // 测试工具Agent
    let tool_response = tool_agent.generate_simple("请查询北京的天气").await?;
    println!("🤖 工具助手: {}", tool_response);

    // 3. 工具组合使用
    println!("\n3️⃣ 工具组合使用");
    println!("----------------");

    let combo_response = tool_agent.generate_simple("请计算 100 + 200，然后告诉我结果").await?;
    println!("🤖 组合工具响应: {}", combo_response);

    println!("\n✅ 工具集成示例完成!");
    println!("\n📚 相关资源:");
    println!("   - examples/02_intermediate/custom_tools.rs - 自定义工具");
    println!("   - examples/04_production/tool_marketplace.rs - 工具市场");
    println!("   - docs/tools/integration.md - 工具集成指南");

    Ok(())
}