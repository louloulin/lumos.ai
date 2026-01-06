//! 多智能体协作模式演示
//!
//! 展示 LumosAI 支持的 12 种协作模式，包括 2025 年最新研究成果
//!
//! 运行方式:
//! ```bash
//! cargo run --package lumosai_examples --example multi_agent_collaboration_demo
//! ```

use lumosai_core::agent::collaboration::AgentRole;
use lumosai_core::agent::{Agent, AgentBuilder, CollaborationMode, Crew};
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 LumosAI 多智能体协作模式演示");
    println!("{}", "=".repeat(60));
    println!();

    // 创建测试 Agents
    let agents = create_demo_agents().await?;

    // 演示所有 12 种协作模式
    demo_basic_modes(&agents).await?;
    demo_sop_modes(&agents).await?;
    demo_advanced_modes(&agents).await?;

    println!("\n✅ 所有协作模式演示完成！");
    Ok(())
}

/// 创建演示用的 Agents
async fn create_demo_agents() -> Result<HashMap<String, Arc<dyn Agent>>, Box<dyn std::error::Error>> {
    let mut agents: HashMap<String, Arc<dyn Agent>> = HashMap::new();

    // 创建 5 个不同角色的 Agent
    let roles = vec![
        ("researcher", "你是一个研究员，擅长收集和分析信息"),
        ("writer", "你是一个作家，擅长创作内容"),
        ("critic", "你是一个评论家，擅长评估和改进内容"),
        ("manager", "你是一个项目经理，擅长规划和协调任务"),
        ("specialist", "你是一个专家，擅长解决特定领域的问题"),
    ];

    for (name, instructions) in roles {
        let llm = create_test_zhipu_provider_arc();
        let agent = AgentBuilder::new()
            .name(name)
            .instructions(instructions)
            .model(llm)
            .build()?;
        agents.insert(name.to_string(), Arc::new(agent) as Arc<dyn Agent>);
    }

    println!("✅ 创建了 {} 个 Agent", agents.len());
    println!();

    Ok(agents)
}

/// 演示基础协作模式
async fn demo_basic_modes(agents: &HashMap<String, Arc<dyn Agent>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 基础协作模式 (3种)");
    println!("{}", "-".repeat(60));

    // 1. Sequential (顺序执行)
    println!("\n1️⃣  Sequential (顺序执行)");
    println!("   描述: Agent 按顺序依次执行任务");
    println!("   场景: 流水线处理、步骤化任务");
    let _crew = create_crew_with_agents("sequential_crew", CollaborationMode::Sequential, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: Sequential)");

    // 2. Parallel (并行执行)
    println!("\n2️⃣  Parallel (并行执行)");
    println!("   描述: 多个 Agent 同时执行任务");
    println!("   场景: 独立任务、并发处理");
    let _crew = create_crew_with_agents("parallel_crew", CollaborationMode::Parallel, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: Parallel)");

    // 3. Hierarchical (层级执行)
    println!("\n3️⃣  Hierarchical (层级执行)");
    println!("   描述: Manager-Worker 层级结构");
    println!("   场景: 团队协作、任务分配");
    let _crew = create_crew_with_agents("hierarchical_crew", CollaborationMode::Hierarchical, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: Hierarchical)");

    println!();
    Ok(())
}

/// 辅助函数：创建 Crew 并添加 Agents
async fn create_crew_with_agents(
    name: &str,
    mode: CollaborationMode,
    agents: &HashMap<String, Arc<dyn Agent>>,
) -> Result<Crew, Box<dyn std::error::Error>> {
    let crew = Crew::new(name.to_string(), mode, 10);

    // 添加所有 agents (使用默认角色)
    for (agent_id, agent) in agents {
        let role = AgentRole::new(
            agent_id.clone(),
            format!("Execute tasks as {}", agent_id),
            format!("{} is a collaborative agent", agent_id),
        );
        crew.add_agent(agent_id.clone(), agent.clone(), role).await?;
    }

    Ok(crew)
}

/// 演示 SOP 执行模式
async fn demo_sop_modes(agents: &HashMap<String, Arc<dyn Agent>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 SOP 执行模式 (3种)");
    println!("{}", "-".repeat(60));

    // 4. SOP React (事件驱动)
    println!("\n4️⃣  SOP React (事件驱动)");
    println!("   描述: Watch-Think-Act 循环");
    println!("   场景: 实时响应、事件处理");
    let _crew = create_crew_with_agents("sop_react_crew", CollaborationMode::SopReact, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: SopReact)");

    // 5. SOP ByOrder (按序执行)
    println!("\n5️⃣  SOP ByOrder (按序执行)");
    println!("   描述: 按照预定义顺序执行");
    println!("   场景: 标准流程、固定步骤");
    let _crew = create_crew_with_agents("sop_byorder_crew", CollaborationMode::SopByOrder, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: SopByOrder)");

    // 6. SOP PlanAndAct (先规划后执行)
    println!("\n6️⃣  SOP PlanAndAct (先规划后执行)");
    println!("   描述: 先制定计划，再执行");
    println!("   场景: 复杂任务、需要规划");
    let _crew = create_crew_with_agents("sop_planandact_crew", CollaborationMode::SopPlanAndAct, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: SopPlanAndAct)");

    println!();
    Ok(())
}

/// 演示高级协作模式 (2025 研究成果)
async fn demo_advanced_modes(agents: &HashMap<String, Arc<dyn Agent>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 高级协作模式 (6种 - 2025 研究成果)");
    println!("{}", "-".repeat(60));

    // 7. Group Chat (群聊协作)
    println!("\n7️⃣  Group Chat (群聊协作)");
    println!("   描述: 多 Agent 轮流发言讨论，达成共识");
    println!("   场景: 头脑风暴、多方讨论、共识决策");
    println!("   来源: Azure AI Agent Orchestration (2025)");
    let _crew = create_crew_with_agents("groupchat_crew", CollaborationMode::GroupChat, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: GroupChat)");

    // 8. Handoff (任务移交)
    println!("\n8️⃣  Handoff (任务移交)");
    println!("   描述: 基于规则的动态任务移交");
    println!("   场景: 客服系统、专家路由、任务分发");
    println!("   来源: Azure AI Agent Orchestration (2025)");
    let _crew = create_crew_with_agents("handoff_crew", CollaborationMode::Handoff, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: Handoff)");

    // 9. Reflection (反思优化)
    println!("\n9️⃣  Reflection (反思优化)");
    println!("   描述: Generator-Critic 双 Agent 迭代改进");
    println!("   场景: 内容创作、代码审查、质量优化");
    println!("   来源: Reflexion, Self-Refine (2023-2024)");
    let _crew = create_crew_with_agents("reflection_crew", CollaborationMode::Reflection, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: Reflection)");

    // 10. Magentic (动态任务规划)
    println!("\n🔟 Magentic (动态任务规划)");
    println!("   描述: Manager-Worker 动态任务分解和分配");
    println!("   场景: 复杂项目管理、动态任务分配");
    println!("   来源: Microsoft Magentic-One (2024)");
    let _crew = create_crew_with_agents("magentic_crew", CollaborationMode::Magentic, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: Magentic)");

    // 11. Debate (多方辩论)
    println!("\n1️⃣1️⃣  Debate (多方辩论)");
    println!("   描述: Proposer-Opposer-Judge 三方辩论");
    println!("   场景: 决策分析、多角度评估、论证验证");
    println!("   来源: Multi-Agent Debate (2023-2025)");
    let _crew = create_crew_with_agents("debate_crew", CollaborationMode::Debate, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: Debate)");

    // 12. MakerChecker (创建-审核)
    println!("\n1️⃣2️⃣  MakerChecker (创建-审核)");
    println!("   描述: Maker-Checker 双阶段质量控制");
    println!("   场景: 内容审核、代码审查、质量控制");
    println!("   来源: 金融行业最佳实践");
    let _crew = create_crew_with_agents("makerchecker_crew", CollaborationMode::MakerChecker, agents).await?;
    println!("   ✅ Crew 创建成功 (模式: MakerChecker)");

    println!();
    Ok(())
}

/// 演示如何切换协作模式
#[allow(dead_code)]
async fn demo_mode_switching(agents: &HashMap<String, Arc<dyn Agent>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 协作模式切换演示");
    println!("{}", "-".repeat(60));

    // 同一组 Agents，不同的协作模式
    let modes = vec![
        (CollaborationMode::Sequential, "Sequential"),
        (CollaborationMode::Parallel, "Parallel"),
        (CollaborationMode::GroupChat, "GroupChat"),
        (CollaborationMode::Reflection, "Reflection"),
    ];

    for (mode, name) in modes {
        println!("\n切换到模式: {}", name);
        let _crew = create_crew_with_agents(name, mode, agents).await?;
        println!("✅ Crew 创建成功");

        // 在实际应用中，这里会调用 crew.kickoff().await
        // let results = crew.kickoff().await?;
    }

    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_demo_agents() {
        let agents = create_demo_agents().await.unwrap();
        assert_eq!(agents.len(), 5);
        assert!(agents.contains_key("researcher"));
        assert!(agents.contains_key("writer"));
        assert!(agents.contains_key("critic"));
        assert!(agents.contains_key("manager"));
        assert!(agents.contains_key("specialist"));
    }

    #[tokio::test]
    async fn test_all_collaboration_modes() {
        let agents = create_demo_agents().await.unwrap();

        // 测试所有 12 种模式都能成功创建 Crew
        let modes = vec![
            (CollaborationMode::Sequential, "sequential"),
            (CollaborationMode::Parallel, "parallel"),
            (CollaborationMode::Hierarchical, "hierarchical"),
            (CollaborationMode::SopReact, "sop_react"),
            (CollaborationMode::SopByOrder, "sop_byorder"),
            (CollaborationMode::SopPlanAndAct, "sop_planandact"),
            (CollaborationMode::GroupChat, "groupchat"),
            (CollaborationMode::Handoff, "handoff"),
            (CollaborationMode::Reflection, "reflection"),
            (CollaborationMode::Magentic, "magentic"),
            (CollaborationMode::Debate, "debate"),
            (CollaborationMode::MakerChecker, "makerchecker"),
        ];

        for (mode, name) in modes {
            let _crew = create_crew_with_agents(name, mode, &agents).await.unwrap();
            // 验证 Crew 创建成功 (通过不 panic 来验证)
        }
    }
}

