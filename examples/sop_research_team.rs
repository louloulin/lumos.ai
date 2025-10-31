//! SOP 研究团队示例
//!
//! 展示如何使用 SOP 机制实现真正的多 Agent 协作
//! 场景：研究员收集信息 → 分析员分析结果 → 撰写员生成报告

use lumosai_core::agent::sop_simple::SimpleSopEnvironment;
use lumosai_core::agent::sop_types::{SopExecutionMode, SopMessage};
use lumosai_core::agent::trait_def::Agent;
use lumosai_core::agent::AgentBuilder;
use lumosai_core::error::Result;
use serde_json::json;
use std::sync::Arc;

/// 创建研究员 Agent
///
/// 研究员关注 "research_request" 消息，执行研究并发送结果
async fn create_researcher() -> Result<Arc<dyn Agent>> {
    let llm = create_test_zhipu_provider_arc();

    let agent = AgentBuilder::new()
        .name("researcher")
        .instructions(
            "You are a research agent. You conduct thorough research on given topics \
             and provide detailed findings.",
        )
        .model(llm)
        .build()?;

    Ok(Arc::new(agent))
}

/// 创建分析员 Agent
///
/// 分析员关注 "research_results" 消息，分析数据并生成洞察
async fn create_analyst() -> Result<Arc<dyn Agent>> {
    let llm = create_test_zhipu_provider_arc();

    let agent = AgentBuilder::new()
        .name("analyst")
        .instructions(
            "You are an analyst agent. You analyze research findings and generate \
             actionable insights and recommendations.",
        )
        .model(llm)
        .build()?;

    Ok(Arc::new(agent))
}

/// 创建撰写员 Agent
///
/// 撰写员关注 "analysis_complete" 消息，生成最终报告
async fn create_writer() -> Result<Arc<dyn Agent>> {
    let llm = create_test_zhipu_provider_arc();

    let agent = AgentBuilder::new()
        .name("writer")
        .instructions(
            "You are a writer agent. You create clear, professional reports based on \
             analysis results.",
        )
        .model(llm)
        .build()?;

    Ok(Arc::new(agent))
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 SOP 研究团队协作示例\n");
    println!("{}", "=".repeat(60));
    println!("场景：研究员 → 分析员 → 撰写员");
    println!("{}", "=".repeat(60));
    println!();

    // 创建 SOP 环境
    let env =
        SimpleSopEnvironment::new("research_team", SopExecutionMode::React).with_max_iterations(20);

    // 创建团队成员
    println!("📋 正在组建研究团队...");
    let researcher = create_researcher().await?;
    let analyst = create_analyst().await?;
    let writer = create_writer().await?;

    // 添加到环境
    env.add_agent(researcher.clone()).await;
    env.add_agent(analyst.clone()).await;
    env.add_agent(writer.clone()).await;

    println!("✅ 团队组建完成！");
    println!("  - Researcher: {}", researcher.get_name());
    println!("  - Analyst: {}", analyst.get_name());
    println!("  - Writer: {}", writer.get_name());
    println!();

    // 说明：由于 BasicAgent 的默认 SOP 方法不参与协作，
    // 这个示例展示了 SOP 基础架构的运行，但 Agent 不会真正响应消息。
    // 要实现真正的协作，需要：
    // 1. 创建自定义 Agent 结构体
    // 2. 实现 sop_watch/think/act 方法
    // 3. 或者使用宏来简化实现
    println!("⚠️  注意：");
    println!("  当前示例使用 BasicAgent，它的默认 SOP 方法不参与协作。");
    println!("  这展示了 SOP 基础架构，但 Agent 不会响应消息。");
    println!("  要实现真正的协作，需要自定义 Agent 实现。");
    println!();

    // 发送初始研究请求
    let initial_msg = SopMessage::broadcast(
        "research_request",
        "user",
        json!({
            "topic": "AI Agent Collaboration Patterns",
            "deadline": "2024-12-31",
            "priority": "high",
            "requirements": [
                "Survey existing approaches",
                "Identify best practices",
                "Analyze performance metrics"
            ]
        }),
    );

    println!("📨 发送研究请求...");
    println!("  主题: AI Agent Collaboration Patterns");
    println!("  优先级: high");
    println!();

    // 运行 SOP 流程
    println!("🔄 开始执行 SOP 流程...");
    println!();

    let stats = env.run(Some(initial_msg)).await?;

    // 显示统计信息
    println!();
    println!("{}", "=".repeat(60));
    println!("📊 执行统计");
    println!("{}", "=".repeat(60));
    println!("总消息数: {}", stats.total_messages);
    println!("活跃 Agent: {}", stats.active_agents);
    println!("完成 Agent: {}", stats.completed_agents);
    println!();

    if !stats.message_types.is_empty() {
        println!("消息类型分布：");
        for (msg_type, count) in &stats.message_types {
            println!("  - {}: {} 条", msg_type, count);
        }
    } else {
        println!("⚠️  没有处理任何消息（Agent 未实现 SOP 方法）");
    }

    println!();
    println!("{}", "=".repeat(60));
    println!("✅ SOP 流程执行完成");
    println!("{}", "=".repeat(60));
    println!();

    // 下一步建议
    println!("💡 下一步：");
    println!("  1. 查看 lumosai_core/src/agent/sop_types.rs 了解 SOP 类型");
    println!("  2. 查看 lumosai_core/src/agent/trait_def.rs 了解 SOP 方法");
    println!("  3. 创建自定义 Agent 实现 sop_watch/think/act 方法");
    println!("  4. 或等待 Agent 宏简化实现（P0-2 任务）");

    Ok(())
}
