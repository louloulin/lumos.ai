//! SOP Agent 协作演示
//!
//! 演示如何使用 SOP 机制实现多 Agent 协作

use lumosai_core::agent::{AgentBuilder, BasicAgent, SopEnvironment, SopExecutionMode, SopMessage};
use lumosai_core::error::Result;
use lumosai_core::llm::MockLlmProvider;
use serde_json::json;
use std::sync::Arc;

/// 研究员 Agent
///
/// 负责接收研究请求并进行研究
struct ResearchAgent {
    inner: BasicAgent,
}

impl ResearchAgent {
    async fn new() -> Result<Self> {
        let llm = Arc::new(MockLlmProvider::new(vec![
            "I have completed the research on AI agents.".to_string(),
        ]));

        let inner = AgentBuilder::new()
            .name("researcher")
            .instructions("You are a research agent. You conduct research on given topics.")
            .model(llm)
            .build()?;

        Ok(Self { inner })
    }
}

/// 分析员 Agent
///
/// 负责分析研究结果
struct AnalystAgent {
    inner: BasicAgent,
}

impl AnalystAgent {
    async fn new() -> Result<Self> {
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Analysis complete: The research shows promising results.".to_string(),
        ]));

        let inner = AgentBuilder::new()
            .name("analyst")
            .instructions("You are an analyst. You analyze research results.")
            .model(llm)
            .build()?;

        Ok(Self { inner })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🤝 SOP Agent 协作演示");
    println!("{}", "=".repeat(50));

    // 创建 SOP 环境
    let env = SopEnvironment::new("research_team", SopExecutionMode::React).with_max_iterations(10);

    println!("\n✅ 创建了 SOP 环境: research_team");
    println!("   执行模式: React (事件驱动)");

    // 创建 Agent
    let researcher = ResearchAgent::new().await?;
    let analyst = AnalystAgent::new().await?;

    println!("\n✅ 创建了 2 个 Agent:");
    println!("   - Researcher: 负责研究");
    println!("   - Analyst: 负责分析");

    // 发布初始消息
    let initial_msg = SopMessage::broadcast(
        "research_request",
        "user",
        json!({
            "topic": "AI Agent Collaboration",
            "deadline": "2024-12-31"
        }),
    );

    println!("\n📨 发布初始消息:");
    println!("   类型: research_request");
    println!("   主题: AI Agent Collaboration");

    // 运行 SOP 流程
    println!("\n🚀 开始执行 SOP 流程...\n");

    let stats = env.run(Some(initial_msg)).await?;

    // 显示统计信息
    println!("\n📊 执行统计:");
    println!("   总消息数: {}", stats.total_messages);
    println!("   活跃 Agent 数: {}", stats.active_agents);
    println!("   完成 Agent 数: {}", stats.completed_agents);

    if !stats.message_types.is_empty() {
        println!("\n   消息类型分布:");
        for (msg_type, count) in &stats.message_types {
            println!("     - {}: {}", msg_type, count);
        }
    }

    println!("\n✅ SOP 流程执行完成！");

    Ok(())
}
