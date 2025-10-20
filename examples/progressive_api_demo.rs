//! 渐进式 API 演示 - LumosAI v2.0 第二周任务
//!
//! 演示三个层次的 Agent API 使用方式：
//! - Level 1: 5分钟上手，智能默认值
//! - Level 2: 链式配置，更多控制
//! - Level 3: 完整构建器模式，高级配置

use lumosai_core::agent::simplified_api::{Agent, quick, web_agent, file_agent, data_agent};
use lumosai_core::agent::trait_def::Agent as AgentTrait;
use lumosai_core::error::Result;
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 LumosAI v2.0 渐进式 API 演示");
    println!("================================");

    // Level 1 API 演示 - 5分钟上手
    println!("\n📚 Level 1 API - 5分钟上手");
    println!("最简单的使用方式，自动配置一切：");

    match Agent::new("assistant", "你是一个友好的AI助手").await {
        Ok(agent) => {
            println!("✅ Agent 创建成功: {}", agent.name());
            println!("📝 指令: {}", agent.instructions());

            // 生成响应
            match agent.generate("你好，请介绍一下自己").await {
                Ok(response) => {
                    println!("🤖 响应: {}", response);
                }
                Err(e) => {
                    println!("❌ 生成响应失败: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Level 1 API 失败 (预期的，因为没有配置模型): {}", e);
            println!("💡 这是正常的，因为自动检测模型需要环境变量配置");
        }
    }

    // Level 2 API 演示 - 链式配置
    println!("\n🔧 Level 2 API - 链式配置");
    println!("提供更多控制，但保持简洁：");

    // 创建一个Mock LLM用于Level 2演示
    let llm_level2 = Arc::new(MockLlmProvider::new(vec![
        "你好！我是你的专业助手，可以根据你的需求定制配置。".to_string(),
        "我现在使用0.7的温度参数，最多可以进行10次工具调用。".to_string(),
    ]));

    let agent_level2 = Agent::new("专业助手", "你是一个友好的AI助手")
        .await
        .expect("创建Level 2基础Agent失败")
        .temperature(0.7)
        .expect("设置温度失败")
        .max_tool_calls(10)
        .expect("设置最大工具调用失败")
        .system_prompt("你是一个专业且友好的助手")
        .expect("设置系统提示失败");

    println!("✅ Level 2 Agent 创建成功: {}", agent_level2.name());
    println!("📝 指令包含'专业': {}", agent_level2.instructions().contains("专业"));
    println!("🎯 温度设置: 0.7 (更创造性)");
    println!("🔧 最大工具调用: 10次");

    // Level 3 API 演示 - 完整构建器模式
    println!("\n⚙️  Level 3 API - 完整构建器模式");
    println!("完全控制，适合高级用户：");

    let llm = Arc::new(MockLlmProvider::new(vec![
        "你好！我是 LumosAI 助手，很高兴为您服务！".to_string(),
        "我可以帮助您处理各种任务，包括回答问题、分析数据、编写代码等。".to_string(),
    ]));

    let agent = Agent::builder()
        .name("advanced_assistant")
        .instructions("你是一个高级AI助手，具备深度思考和分析能力")
        .model(llm)
        .max_tool_calls(10)
        .temperature(0.7)
        .build()?;

    println!("✅ 高级 Agent 创建成功: {}", agent.get_name());
    println!("📝 指令: {}", agent.get_instructions());

    // 使用 Agent trait 的方法
    let response = agent.generate_simple("请详细介绍你的能力").await?;
    println!("🤖 响应: {}", response);

    // 演示快速创建方法
    println!("\n🚀 快速创建方法演示");
    let quick_agent = Agent::quick("quick_helper", "你是一个快速助手")
        .model(Arc::new(MockLlmProvider::new(vec![
            "我是快速助手，随时为您提供帮助！".to_string(),
        ])))
        .build()?;

    println!("✅ 快速 Agent 创建成功: {}", quick_agent.get_name());
    let quick_response = quick_agent.generate_simple("你好").await?;
    println!("🤖 快速响应: {}", quick_response);

    // 便捷函数演示
    println!("\n🛠️  便捷函数演示");
    println!("针对特定场景的快速配置：");

    let llm_convenience = Arc::new(MockLlmProvider::new(vec![
        "我是Web助手，可以帮您处理网页相关任务。".to_string(),
        "我是文件助手，可以帮您管理文件和目录。".to_string(),
        "我是数据助手，可以帮您处理和分析数据。".to_string(),
    ]));

    // Web助手
    let web_helper = web_agent("web助手")
        .model(llm_convenience.clone())
        .build()
        .expect("创建Web助手失败");
    println!("✅ Web助手创建成功: {}", web_helper.get_name());

    // 文件助手
    let file_helper = file_agent("文件助手")
        .model(llm_convenience.clone())
        .build()
        .expect("创建文件助手失败");
    println!("✅ 文件助手创建成功: {}", file_helper.get_name());

    // 数据助手
    let data_helper = data_agent("数据助手")
        .model(llm_convenience)
        .build()
        .expect("创建数据助手失败");
    println!("✅ 数据助手创建成功: {}", data_helper.get_name());

    println!("\n🎉 渐进式 API 演示完成！");
    println!("📊 功能总结：");
    println!("  - Level 1: 零配置智能默认值，适合快速原型");
    println!("  - Level 2: 链式配置API，支持温度、工具调用等设置");
    println!("  - Level 3: 完整构建器模式，完全控制所有配置");
    println!("  - 便捷函数: 特定场景的快速创建（web、文件、数据助手）");
    
    println!("\n💡 使用建议：");
    println!("  - 初学者: 使用 Level 1 API 快速上手");
    println!("  - 中级用户: 使用 Level 2 API 进行定制");
    println!("  - 高级用户: 使用 Level 3 API 完全控制");
    println!("  - 特定需求: 使用便捷函数快速创建专业助手");

    Ok(())
}
