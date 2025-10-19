//! 多 Agent 协作演示程序
//!
//! 演示如何使用 Crew 和 Task 实现多 Agent 协作，对标 CrewAI

use lumosai_core::agent::{
    AgentBuilder, AgentTask, CollaborationMode, Crew, CrewAgentRole, TaskStatus,
};
use lumosai_core::llm::MockLlmProvider;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤝 多 Agent 协作演示 - Multi-Agent Collaboration\n");
    println!("{}", "=".repeat(80));
    println!();

    // 演示 1: 创建 Agent 团队
    demo_create_crew().await?;

    // 演示 2: 顺序执行任务
    demo_sequential_execution().await?;

    // 演示 3: 并行执行任务
    demo_parallel_execution().await?;

    // 演示 4: 层级执行任务
    demo_hierarchical_execution().await?;

    // 总结
    println!("\n{}", "=".repeat(80));
    println!("✅ 多 Agent 协作演示完成！");
    println!("{}", "=".repeat(80));
    println!();
    println!("📊 功能总结:");
    println!("  1. Agent 团队管理: 创建和管理 Agent 团队");
    println!("  2. 任务分配: 自动分配任务给合适的 Agent");
    println!("  3. 协作模式: 顺序、并行、层级三种执行模式");
    println!("  4. 依赖管理: 支持任务依赖关系");
    println!("  5. 并发控制: 限制同时执行的任务数量");
    println!();
    println!("💡 技术优势:");
    println!("  - 类型安全: Rust 类型系统保证协作正确性");
    println!("  - 高性能: 异步并发执行，无 GIL 限制");
    println!("  - 灵活配置: 支持多种协作模式");
    println!("  - 可扩展性: 易于添加新的 Agent 和任务");
    println!();
    println!("🚀 对标 CrewAI:");
    println!("  ✅ Agent 角色定义");
    println!("  ✅ 任务分配和协调");
    println!("  ✅ 多种协作模式");
    println!("  ✅ 任务依赖管理");
    println!("  ✅ 并发控制");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// 演示 1: 创建 Agent 团队
async fn demo_create_crew() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 演示 1: 创建 Agent 团队");
    println!("{}", "-".repeat(80));

    // 创建团队
    let crew = Crew::new(
        "Research Team".to_string(),
        CollaborationMode::Sequential,
        3, // 最大并发任务数
    );

    println!("✅ 创建团队: Research Team");
    println!("  - 协作模式: Sequential（顺序执行）");
    println!("  - 最大并发数: 3");
    println!();

    // 创建 Agent 1: 研究员
    let researcher_llm = Arc::new(MockLlmProvider::new(vec![
        "I'll research this topic thoroughly.".to_string(),
    ]));
    let researcher = AgentBuilder::new()
        .name("researcher")
        .instructions("You are a research expert")
        .model(researcher_llm)
        .build()?;

    let researcher_role = CrewAgentRole::new(
        "Researcher".to_string(),
        "Research and analyze topics".to_string(),
        "Expert researcher with deep analytical skills".to_string(),
    );

    crew.add_agent(
        "researcher".to_string(),
        Arc::new(researcher),
        researcher_role,
    )
    .await?;

    println!("✅ 添加 Agent 1: Researcher");
    println!("  - 角色: 研究员");
    println!("  - 目标: 研究和分析主题");
    println!("  - 背景: 具有深度分析能力的专家研究员");
    println!();

    // 创建 Agent 2: 作家
    let writer_llm = Arc::new(MockLlmProvider::new(vec![
        "I'll write engaging content based on the research.".to_string(),
    ]));
    let writer = AgentBuilder::new()
        .name("writer")
        .instructions("You are a content writer")
        .model(writer_llm)
        .build()?;

    let writer_role = CrewAgentRole::new(
        "Writer".to_string(),
        "Write engaging content".to_string(),
        "Professional writer with excellent communication skills".to_string(),
    );

    crew.add_agent("writer".to_string(), Arc::new(writer), writer_role)
        .await?;

    println!("✅ 添加 Agent 2: Writer");
    println!("  - 角色: 作家");
    println!("  - 目标: 编写引人入胜的内容");
    println!("  - 背景: 具有出色沟通技巧的专业作家");
    println!();

    // 创建 Agent 3: 编辑
    let editor_llm = Arc::new(MockLlmProvider::new(vec![
        "I'll review and polish the content.".to_string(),
    ]));
    let editor = AgentBuilder::new()
        .name("editor")
        .instructions("You are an editor")
        .model(editor_llm)
        .build()?;

    let editor_role = CrewAgentRole::new(
        "Editor".to_string(),
        "Review and polish content".to_string(),
        "Experienced editor with attention to detail".to_string(),
    );

    crew.add_agent("editor".to_string(), Arc::new(editor), editor_role)
        .await?;

    println!("✅ 添加 Agent 3: Editor");
    println!("  - 角色: 编辑");
    println!("  - 目标: 审查和润色内容");
    println!("  - 背景: 注重细节的资深编辑");
    println!();

    println!("💡 团队创建完成！共 3 个 Agent");
    println!();

    Ok(())
}

/// 演示 2: 顺序执行任务
async fn demo_sequential_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 演示 2: 顺序执行任务 (Sequential)");
    println!("{}", "-".repeat(80));

    // 创建团队
    let crew = Crew::new(
        "Content Team".to_string(),
        CollaborationMode::Sequential,
        1,
    );

    // 添加 Agent
    let llm = Arc::new(MockLlmProvider::new(vec!["Done".to_string()]));
    let agent = AgentBuilder::new()
        .name("agent1")
        .instructions("You are a helpful agent")
        .model(llm)
        .build()?;

    crew.add_agent(
        "agent1".to_string(),
        Arc::new(agent),
        CrewAgentRole::new(
            "Worker".to_string(),
            "Complete tasks".to_string(),
            "Hardworking agent".to_string(),
        ),
    )
    .await?;

    // 创建任务
    let task1 = AgentTask::new("Research AI trends".to_string())
        .with_expected_output("Research report".to_string())
        .with_priority(8);

    let task2 = AgentTask::new("Write article about AI".to_string())
        .with_expected_output("Article draft".to_string())
        .with_priority(7)
        .with_dependency(task1.id.clone());

    let task3 = AgentTask::new("Edit and publish article".to_string())
        .with_expected_output("Published article".to_string())
        .with_priority(6)
        .with_dependency(task2.id.clone());

    println!("📋 创建 3 个任务:");
    println!("  1. {} (优先级: 8)", task1.description);
    println!("  2. {} (优先级: 7, 依赖: Task 1)", task2.description);
    println!("  3. {} (优先级: 6, 依赖: Task 2)", task3.description);
    println!();

    crew.add_task(task1).await?;
    crew.add_task(task2).await?;
    crew.add_task(task3).await?;

    println!("🚀 开始顺序执行...");
    let results = crew.kickoff().await?;

    println!("\n✅ 执行完成！结果:");
    for (i, task) in results.iter().enumerate() {
        println!("  {}. {} - 状态: {:?}", i + 1, task.description, task.status);
        if let Some(result) = &task.result {
            println!("     结果: {}", result);
        }
    }

    let stats = crew.get_stats().await;
    println!("\n📊 统计信息:");
    println!("  - 总任务数: {}", stats.total_tasks);
    println!("  - 已完成: {}", stats.completed_tasks);
    println!("  - 失败: {}", stats.failed_tasks);
    println!();

    Ok(())
}

/// 演示 3: 并行执行任务
async fn demo_parallel_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚡ 演示 3: 并行执行任务 (Parallel)");
    println!("{}", "-".repeat(80));

    // 创建团队（并行模式）
    let crew = Crew::new(
        "Parallel Team".to_string(),
        CollaborationMode::Parallel,
        3, // 允许 3 个任务并行
    );

    // 添加 Agent
    let llm = Arc::new(MockLlmProvider::new(vec!["Done".to_string()]));
    let agent = AgentBuilder::new()
        .name("agent1")
        .instructions("You are a helpful agent")
        .model(llm)
        .build()?;

    crew.add_agent(
        "agent1".to_string(),
        Arc::new(agent),
        CrewAgentRole::new(
            "Worker".to_string(),
            "Complete tasks".to_string(),
            "Hardworking agent".to_string(),
        ),
    )
    .await?;

    // 创建独立任务（无依赖）
    let task1 = AgentTask::new("Analyze market data".to_string()).with_priority(8);
    let task2 = AgentTask::new("Generate report".to_string()).with_priority(7);
    let task3 = AgentTask::new("Send notifications".to_string()).with_priority(6);

    println!("📋 创建 3 个独立任务:");
    println!("  1. {} (优先级: 8)", task1.description);
    println!("  2. {} (优先级: 7)", task2.description);
    println!("  3. {} (优先级: 6)", task3.description);
    println!();

    crew.add_task(task1).await?;
    crew.add_task(task2).await?;
    crew.add_task(task3).await?;

    println!("🚀 开始并行执行（最多 3 个任务同时运行）...");
    let results = crew.kickoff().await?;

    println!("\n✅ 执行完成！结果:");
    for (i, task) in results.iter().enumerate() {
        println!("  {}. {} - 状态: {:?}", i + 1, task.description, task.status);
    }

    println!("\n💡 并行执行优势:");
    println!("  - 提高效率: 多个任务同时执行");
    println!("  - 资源利用: 充分利用多核 CPU");
    println!("  - 并发控制: 限制最大并发数避免资源耗尽");
    println!();

    Ok(())
}

/// 演示 4: 层级执行任务
async fn demo_hierarchical_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏢 演示 4: 层级执行任务 (Hierarchical)");
    println!("{}", "-".repeat(80));

    // 创建团队（层级模式）
    let crew = Crew::new(
        "Hierarchical Team".to_string(),
        CollaborationMode::Hierarchical,
        2,
    );

    // 添加管理者 Agent
    let manager_llm = Arc::new(MockLlmProvider::new(vec!["Task assigned".to_string()]));
    let manager = AgentBuilder::new()
        .name("manager")
        .instructions("You are a team manager")
        .model(manager_llm)
        .build()?;

    crew.add_agent(
        "manager".to_string(),
        Arc::new(manager),
        CrewAgentRole::new(
            "Manager".to_string(),
            "Coordinate team tasks".to_string(),
            "Experienced team manager".to_string(),
        )
        .with_delegation(),
    )
    .await?;

    // 添加工作者 Agent
    let worker_llm = Arc::new(MockLlmProvider::new(vec!["Task completed".to_string()]));
    let worker = AgentBuilder::new()
        .name("worker")
        .instructions("You are a worker")
        .model(worker_llm)
        .build()?;

    crew.add_agent(
        "worker".to_string(),
        Arc::new(worker),
        CrewAgentRole::new(
            "Worker".to_string(),
            "Execute tasks".to_string(),
            "Reliable worker".to_string(),
        ),
    )
    .await?;

    // 创建任务
    let task1 = AgentTask::new("Plan project".to_string()).with_priority(10);
    let task2 = AgentTask::new("Execute project".to_string())
        .with_priority(8)
        .with_dependency(task1.id.clone());

    println!("📋 创建 2 个任务:");
    println!("  1. {} (优先级: 10)", task1.description);
    println!("  2. {} (优先级: 8, 依赖: Task 1)", task2.description);
    println!();

    crew.add_task(task1).await?;
    crew.add_task(task2).await?;

    println!("🚀 开始层级执行（由管理者协调）...");
    let results = crew.kickoff().await?;

    println!("\n✅ 执行完成！结果:");
    for (i, task) in results.iter().enumerate() {
        println!("  {}. {} - 状态: {:?}", i + 1, task.description, task.status);
    }

    println!("\n💡 层级执行特点:");
    println!("  - 管理者协调: 由管理者 Agent 分配和协调任务");
    println!("  - 任务委托: 管理者可以将任务委托给工作者");
    println!("  - 清晰层级: 明确的管理和执行层级");
    println!();

    Ok(())
}

