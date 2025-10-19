//! 渐进式 API 演示 - LumosAI v2.0 第二周任务
//!
//! 演示三个层次的 Agent API 使用方式：
//! - Level 1: 5分钟上手，智能默认值
//! - Level 2: 链式配置，更多控制
//! - Level 3: 完整构建器模式，高级配置

use lumosai_core::agent::simplified_api::Agent;
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

    // Level 2 API 演示 - 链式配置（模拟）
    println!("\n🔧 Level 2 API - 链式配置");
    println!("提供更多控制，但保持简洁：");

    // 注意：由于当前实现的限制，Level 2 API 暂时返回错误
    // 这是设计上的权衡，真正的 Level 2 API 需要更复杂的内部架构
    println!("💡 Level 2 API 当前返回配置错误，建议使用 Level 3 API");

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

    println!("\n🎉 渐进式 API 演示完成！");
    println!("📊 总结：");
    println!("  - Level 1: 最简单，自动配置（需要环境变量）");
    println!("  - Level 2: 链式配置（当前版本有限制）");
    println!("  - Level 3: 完全控制，适合生产环境");

    Ok(())
}
