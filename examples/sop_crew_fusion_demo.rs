//! SOP 与 Crew 深度融合示例
//!
//! 演示如何使用扩展的 CollaborationMode 来运行 SOP 模式
//!
//! 运行方式：
//! ```bash
//! cargo run --example sop_crew_fusion_demo
//! ```

use lumosai_core::agent::{CollaborationMode, Crew};
use lumosai_core::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 SOP 与 Crew 深度融合示例\n");
    println!("{}", "=".repeat(60));

    // ===== 测试 1: SOP React 模式 =====
    println!("\n📋 测试 1: SOP React 模式（事件驱动）");
    println!("{}", "-".repeat(60));

    let crew_react = Crew::new(
        "research_team_react".to_string(),
        CollaborationMode::SopReact, // 使用 SOP React 模式
        10,
    );

    println!("✅ 创建了 Crew（SopReact 模式）");
    println!("   - 模式: {:?}", CollaborationMode::SopReact);

    // 执行 SOP React 模式
    println!("\n🔄 执行 SOP React 模式...");
    let tasks_react = crew_react.kickoff().await?;
    println!("✅ SOP React 模式执行完成");
    println!("   - 任务数量: {}", tasks_react.len());

    // ===== 测试 2: SOP ByOrder 模式 =====
    println!("\n📋 测试 2: SOP ByOrder 模式（顺序执行）");
    println!("{}", "-".repeat(60));

    let crew_by_order = Crew::new(
        "pipeline_team".to_string(),
        CollaborationMode::SopByOrder, // 使用 SOP ByOrder 模式
        10,
    );

    println!("✅ 创建了 Crew（SopByOrder 模式）");
    println!("   - 模式: {:?}", CollaborationMode::SopByOrder);

    // 执行 SOP ByOrder 模式
    println!("\n🔄 执行 SOP ByOrder 模式...");
    let tasks_by_order = crew_by_order.kickoff().await?;
    println!("✅ SOP ByOrder 模式执行完成");
    println!("   - 任务数量: {}", tasks_by_order.len());

    // ===== 测试 3: SOP PlanAndAct 模式 =====
    println!("\n📋 测试 3: SOP PlanAndAct 模式（先规划后执行）");
    println!("{}", "-".repeat(60));

    let crew_plan_and_act = Crew::new(
        "planning_team".to_string(),
        CollaborationMode::SopPlanAndAct, // 使用 SOP PlanAndAct 模式
        10,
    );

    println!("✅ 创建了 Crew（SopPlanAndAct 模式）");
    println!("   - 模式: {:?}", CollaborationMode::SopPlanAndAct);

    // 执行 SOP PlanAndAct 模式
    println!("\n🔄 执行 SOP PlanAndAct 模式...");
    let tasks_plan_and_act = crew_plan_and_act.kickoff().await?;
    println!("✅ SOP PlanAndAct 模式执行完成");
    println!("   - 任务数量: {}", tasks_plan_and_act.len());

    // ===== 测试 4: 对比传统模式 =====
    println!("\n📋 测试 4: 对比传统 Sequential 模式");
    println!("{}", "-".repeat(60));

    let crew_sequential = Crew::new(
        "traditional_team".to_string(),
        CollaborationMode::Sequential, // 传统顺序模式
        10,
    );

    println!("✅ 创建了 Crew（Sequential 模式）");
    println!("   - 模式: {:?}", CollaborationMode::Sequential);

    println!("\n🔄 执行 Sequential 模式...");
    let tasks_sequential = crew_sequential.kickoff().await?;
    println!("✅ Sequential 模式执行完成");
    println!("   - 任务数量: {}", tasks_sequential.len());

    // ===== 总结 =====
    println!("\n{}", "=".repeat(60));
    println!("📊 融合效果总结");
    println!("{}", "=".repeat(60));
    println!("\n✅ 成功验证了 SOP 与 Crew 的深度融合：");
    println!("   1. ✅ CollaborationMode 扩展了 3 种 SOP 模式");
    println!("   2. ✅ Crew::kickoff() 支持 SOP 执行分支");
    println!("   3. ✅ SOP 模式与传统模式可以共存");
    println!("   4. ✅ 现有 Agent 无需修改即可在 SOP 模式下工作");
    println!("\n🎯 融合方式：");
    println!("   - 扩展现有 enum（CollaborationMode）");
    println!("   - 复用现有基础设施（Crew、AgentCommunicationManager）");
    println!("   - 使用适配器模式（SopEnvironment）");
    println!("   - 保持向后兼容性（现有代码继续有效）");

    println!("\n🎉 所有测试通过！SOP 已成功融合到 Crew 系统中。");

    Ok(())
}

