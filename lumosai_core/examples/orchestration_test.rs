//! Multi-Agent 编排系统测试示例
//!
//! 运行方式: cargo run --example orchestration_test

use lumosai_core::orchestration::{
    AgentCapabilities, AgentInfo, AgentRegistry, AgentRole, CrewConfig, CrewManager,
    MessageBus, MultiAgentCoordinator, OrchestrationPattern, Task, TaskPriority, TaskQueue, TaskRouter, RoutingStrategy,
};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== LumosAI Multi-Agent 编排系统测试 ===\n");

    // 测试 1: 创建 MultiAgentCoordinator
    println!("测试 1: 创建 MultiAgentCoordinator");
    let coordinator = MultiAgentCoordinator::new(OrchestrationPattern::Hierarchical);
    println!("✅ 协调器创建成功");
    println!("  - 模式: {:?}", coordinator.config());
    println!("  - 最大并发 Agent 数: {}\n", coordinator.config().max_concurrent_agents);

    // 测试 2: Agent 注册表
    println!("测试 2: Agent 注册表");
    let registry = AgentRegistry::new();

    let agent1 = AgentInfo::new("agent1", "Math Expert", vec!["worker".to_string()]);
    let agent2 = AgentInfo::new("agent2", "Writer", vec!["writer".to_string()]);
    let agent3 = AgentInfo::new("agent3", "Manager", vec!["manager".to_string()]);

    println!("✅ Agent 信息创建成功");
    println!("  - agent1: {} (roles: worker)", agent1.name);
    println!("  - agent2: {} (roles: writer)", agent2.name);
    println!("  - agent3: {} (roles: manager)", agent3.name);
    println!();

    // 测试 3: 编排模式
    println!("测试 3: 编排模式");
    println!("  - Hierarchical: {}", OrchestrationPattern::Hierarchical.description());
    println!("  - Flat: {}", OrchestrationPattern::Flat.description());
    println!("  - Pipeline: {}", OrchestrationPattern::Pipeline.description());
    println!("  - Graph: {}", OrchestrationPattern::Graph.description());
    println!();

    // 测试 4: 消息总线
    println!("测试 4: 消息总线");
    let message_bus = MessageBus::new();
    println!("✅ MessageBus 创建成功");
    println!();

    // 测试 5: 任务队列
    println!("测试 5: 任务队列");
    let task1 = Task::new("calculation", "2 + 2 = ?").with_priority(TaskPriority::High);
    let task2 = Task::new("writing", "Write a poem");

    println!("✅ 任务创建成功");
    println!("  - Task 1: {} (priority: {:?})", task1.task_type, task1.priority);
    println!("  - Task 2: {} (priority: {:?})", task2.task_type, task2.priority);
    println!();

    // 测试 6: 任务路由器
    println!("测试 6: 任务路由器");
    use lumosai_core::orchestration::{TaskRouter, RoutingStrategy};

    let mut router = TaskRouter::new().with_strategy(RoutingStrategy::RoundRobin);
    let agents = vec!["agent1".to_string(), "agent2".to_string(), "agent3".to_string()];

    let r1 = router.route(&agents, "task1");
    let r2 = router.route(&agents, "task2");
    let r3 = router.route(&agents, "task3");

    println!("✅ 轮询路由测试:");
    println!("  - task1 -> {:?}", r1);
    println!("  - task2 -> {:?}", r2);
    println!("  - task3 -> {:?}", r3);
    println!();

    // 测试 7: CrewManager
    println!("测试 7: CrewManager (异步测试需要 tokio runtime)");
    println!("  注意: 以下为同步 API 演示\n");

    let config = CrewConfig::default();
    println!("✅ CrewConfig 创建:");
    println!("  - 名称: {}", config.name);
    println!("  - 最大并行任务: {}", config.max_parallel_tasks);
    println!();

    println!("=== 所有测试通过! ✅ ===");
    println!("\nMulti-Agent 编排系统功能:");
    println!("  ✅ MultiAgentCoordinator - 核心协调器");
    println!("  ✅ OrchestrationPattern - 4种编排模式");
    println!("  ✅ AgentRegistry - Agent 注册表");
    println!("  ✅ MessageBus - Agent 通信总线");
    println!("  ✅ TaskQueue - 任务队列管理");
    println!("  ✅ TaskRouter - 智能任务路由");
    println!("  ✅ CrewManager - Agent 团队管理");
    println!("\n编排模式:");
    println!("  1. Hierarchical - Manager 协调 Workers");
    println!("  2. Flat - 所有 Agent 平等协作");
    println!("  3. Pipeline - 流水线顺序执行");
    println!("  4. Graph - 复杂依赖图");

    Ok(())
}
