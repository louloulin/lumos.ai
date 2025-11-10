//! MVP 示例 5: 简单的工作流模式
//!
//! 这个示例展示如何使用多个 Agent 按顺序协作完成复杂任务（工作流模式）。
//!
//! 运行方式:
//! ```bash
//! cargo run --package lumosai_examples --example mvp_05_workflow
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 5: 简单的工作流模式\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 创建三个专门化的 Agents 组成工作流
    println!("\n📝 步骤 1: 创建工作流 Agents");

    let llm1 = create_test_zhipu_provider_arc();
    let planner = AgentBuilder::new()
        .name("planner")
        .instructions("You are a project planner. Create clear, actionable project plans with specific steps.")
        .model(llm1)
        .build()?;
    println!("✅ Planner Agent '{}' 创建成功", planner.name().unwrap_or("unknown"));

    let llm2 = create_test_zhipu_provider_arc();
    let executor = AgentBuilder::new()
        .name("executor")
        .instructions("You are a project executor. Follow the plan and describe how you would execute each step.")
        .model(llm2)
        .build()?;
    println!("✅ Executor Agent '{}' 创建成功", executor.name().unwrap_or("unknown"));

    let llm3 = create_test_zhipu_provider_arc();
    let reviewer = AgentBuilder::new()
        .name("reviewer")
        .instructions("You are a project reviewer. Review the execution and provide constructive feedback.")
        .model(llm3)
        .build()?;
    println!("✅ Reviewer Agent '{}' 创建成功", reviewer.name().unwrap_or("unknown"));

    // 步骤 2: 可视化工作流结构
    println!("\n📝 步骤 2: 工作流结构");
    println!("{}", "━".repeat(50));
    println!("\n   📊 工作流流程:");
    println!();
    println!("      ┌──────────┐");
    println!("      │ Planner  │  制定计划");
    println!("      └────┬─────┘");
    println!("           │");
    println!("           ▼");
    println!("      ┌──────────┐");
    println!("      │ Executor │  执行任务");
    println!("      └────┬─────┘");
    println!("           │");
    println!("           ▼");
    println!("      ┌──────────┐");
    println!("      │ Reviewer │  审查结果");
    println!("      └──────────┘");
    println!();

    // 步骤 3: 设置任务并执行工作流
    println!("📝 步骤 3: 执行工作流");
    println!("{}", "━".repeat(50));

    let project = "Build a simple todo list application";
    println!("\n📌 项目任务: {}\n", project);

    // 阶段 1: 规划
    println!("🔵 阶段 1: 规划");
    let plan_prompt = format!("Create a detailed plan for: {}", project);
    let plan = planner.generate_simple(&plan_prompt).await?;
    println!("📋 计划:\n{}\n", plan);

    // 阶段 2: 执行
    println!("🟢 阶段 2: 执行");
    let execute_prompt = format!("Based on this plan:\n{}\n\nDescribe how you would execute it.", plan);
    let execution = executor.generate_simple(&execute_prompt).await?;
    println!("⚙️  执行:\n{}\n", execution);

    // 阶段 3: 审查
    println!("🟡 阶段 3: 审查");
    let review_prompt = format!(
        "Review this project execution:\n\nPlan:\n{}\n\nExecution:\n{}\n\nProvide feedback.",
        plan, execution
    );
    let review = reviewer.generate_simple(&review_prompt).await?;
    println!("📝 审查:\n{}\n", review);

    // 总结
    println!("{}", "=".repeat(50));
    println!("✅ 工作流执行完成！");
    println!("\n💡 关键概念:");
    println!("   - 工作流：多个 Agent 按顺序协作");
    println!("   - 每个 Agent 专注于特定任务");
    println!("   - 前一个 Agent 的输出作为下一个的输入");
    println!("   - 通过协作完成复杂任务");
    println!("\n🔄 工作流模式:");
    println!("   ✓ 顺序执行：一个接一个完成任务");
    println!("   ✓ 数据流动：信息在 Agent 之间传递");
    println!("   ✓ 专业分工：每个 Agent 有明确的职责");
    println!("   ✓ 可扩展性：可以添加更多 Agent");
    println!("\n🎯 实际应用:");
    println!("   - 内容创作流程（研究 → 写作 → 编辑）");
    println!("   - 软件开发流程（设计 → 实现 → 测试）");
    println!("   - 数据处理流程（收集 → 清洗 → 分析）");
    println!("   - 决策支持流程（分析 → 建议 → 审核）");
    println!("\n🚀 高级工作流功能:");
    println!("   - DAG Workflow：支持复杂的依赖关系");
    println!("   - 并行执行：独立任务可以同时运行");
    println!("   - 条件分支：根据结果选择不同路径");
    println!("   - 错误恢复：自动重试和故障处理");
    println!("\n🎓 恭喜完成所有 MVP 示例！");
    println!("   你已经掌握了 LumosAI 的核心功能：");
    println!("   ✅ 创建基本 Agent");
    println!("   ✅ 使用工具扩展能力");
    println!("   ✅ 多 Agent 协作");
    println!("   ✅ 上下文对话");
    println!("   ✅ 工作流编排");
    println!("\n   现在可以开始构建自己的 AI 应用了！🚀");

    Ok(())
}
