//! MVP 示例 5: Workflow 工作流编排
//!
//! 这个示例展示如何使用 Workflow 系统编排复杂的 Agent 任务流程。
//!
//! 运行方式:
//! ```bash
//! cargo run --package lumosai_examples --example mvp_05_workflow
//! ```

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::agent::types::RuntimeContext;
use lumosai_core::base::Base;
use lumosai_core::llm::test_helpers::create_test_zhipu_provider_arc;
use lumosai_core::workflow::{DagWorkflow, DagWorkflowBuilder};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 MVP 示例 5: Workflow 工作流编排\n");
    println!("{}", "=".repeat(50));

    // 步骤 1: 创建多个 Agent 用于不同任务
    println!("\n📝 步骤 1: 创建任务 Agents");

    let llm1 = create_test_zhipu_provider_arc();
    let planner = AgentBuilder::new()
        .name("planner")
        .instructions("You are a project planner. Create clear project plans with milestones and tasks.")
        .model(llm1)
        .build()?;
    println!("✅ Planner Agent 创建成功");

    let llm2 = create_test_zhipu_provider_arc();
    let executor = AgentBuilder::new()
        .name("executor")
        .instructions("You are a project executor. Execute tasks according to the plan.")
        .model(llm2)
        .build()?;
    println!("✅ Executor Agent 创建成功");

    let llm3 = create_test_zhipu_provider_arc();
    let reviewer = AgentBuilder::new()
        .name("reviewer")
        .instructions("You are a project reviewer. Review execution results and provide feedback.")
        .model(llm3)
        .build()?;
    println!("✅ Reviewer Agent 创建成功");

    // 步骤 2: 创建 DAG Workflow
    println!("\n📝 步骤 2: 创建 DAG Workflow");
    let mut workflow = DagWorkflowBuilder::new("project_workflow")
        .description("A complete project execution workflow with planning, execution, and review")
        .build();
    println!("✅ DAG Workflow '{}' 创建成功", workflow.name);

    // 步骤 3: 添加节点（Agents）到工作流
    println!("\n📝 步骤 3: 添加 Agent 节点");
    
    workflow.add_agent_node("plan", planner)?;
    println!("   ➕ 添加节点: plan");
    
    workflow.add_agent_node("execute", executor)?;
    println!("   ➕ 添加节点: execute");
    
    workflow.add_agent_node("review", reviewer)?;
    println!("   ➕ 添加节点: review");

    // 步骤 4: 设置节点依赖关系（DAG 边）
    println!("\n📝 步骤 4: 设置节点依赖");
    
    workflow.add_dependency("execute", "plan")?;
    println!("   🔗 execute 依赖 plan");
    
    workflow.add_dependency("review", "execute")?;
    println!("   🔗 review 依赖 execute");

    // 步骤 5: 可视化工作流结构
    println!("\n📝 步骤 5: 工作流结构");
    println!("{}", "━".repeat(50));
    println!("\n   📊 DAG 结构:");
    println!("   ");
    println!("      ┌─────────┐");
    println!("      │  plan   │");
    println!("      └────┬────┘");
    println!("           │");
    println!("           ▼");
    println!("      ┌─────────┐");
    println!("      │ execute │");
    println!("      └────┬────┘");
    println!("           │");
    println!("           ▼");
    println!("      ┌─────────┐");
    println!("      │ review  │");
    println!("      └─────────┘");
    println!();

    // 步骤 6: 设置初始输入和上下文
    println!("📝 步骤 6: 设置初始输入");
    let initial_input = "Build a simple AI chatbot application";
    println!("   📌 项目: {}", initial_input);
    
    let input_value = json!({
        "task": initial_input,
        "requirements": "The chatbot should be user-friendly and efficient"
    });
    
    let context = RuntimeContext::new();

    // 步骤 7: 执行工作流
    println!("\n📝 步骤 7: 执行 Workflow");
    println!("{}", "━".repeat(50));

    println!("\n🔄 开始执行工作流...\n");
    
    match workflow.execute_dag(input_value, &context).await {
        Ok(result) => {
            println!("✅ Workflow 执行成功！\n");
            println!("📊 执行结果:");
            println!("{}", "━".repeat(50));
            println!("\n{}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            eprintln!("❌ Workflow 执行失败: {}", e);
            eprintln!("   这是正常的，因为我们使用的是测试 LLM 提供商");
            eprintln!("   在实际应用中，请使用真实的 LLM API");
        }
    }

    println!("\n{}", "=".repeat(50));
    println!("✅ Workflow 示例完成！");
    println!("\n💡 关键概念:");
    println!("   - DAG (有向无环图) Workflow 结构");
    println!("   - 节点：代表要执行的任务（Agent）");
    println!("   - 边：代表任务之间的依赖关系");
    println!("   - 自动按依赖顺序执行任务");
    println!("   - 支持并行执行独立任务");
    println!("\n🔧 Workflow 特性:");
    println!("   ✓ DAG 构建和验证（环检测）");
    println!("   ✓ 拓扑排序执行");
    println!("   ✓ 并行执行（独立节点）");
    println!("   ✓ 上下文传递（节点间数据流）");
    println!("   ✓ 错误处理和恢复");
    println!("\n🎯 适用场景:");
    println!("   - 复杂的多步骤任务");
    println!("   - 需要明确依赖关系的流程");
    println!("   - 需要并行处理的任务");
    println!("   - 可重复的工作流程");
    println!("\n🎓 恭喜完成所有 MVP 示例！");
    println!("   现在你已经掌握了 LumosAI 的核心功能。");
    println!("   可以开始构建自己的 AI 应用了！");

    Ok(())
}

