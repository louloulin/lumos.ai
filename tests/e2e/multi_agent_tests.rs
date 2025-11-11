//! E2E 测试：Multi-Agent 协作场景

mod framework;
use framework::{E2ETestContext, E2EAssertions};

use lumosai_core::agent::collaboration::AgentRole;
use lumosai_core::agent::{Agent, AgentChain, AgentParallel, CollaborationMode, Crew};
use std::sync::Arc;

/// 测试 18: Agent Chain 顺序执行
#[tokio::test]
async fn test_agent_chain_sequential() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建多个 Agent
    let agent1 = ctx
        .create_test_agent("analyzer", "Analyze the input and provide key points.")
        .unwrap();

    let agent2 = ctx
        .create_test_agent("summarizer", "Summarize the key points concisely.")
        .unwrap();

    // 创建 Agent Chain (使用新 API)
    let mut chain = AgentChain::new();
    chain.add_agent("analyzer".to_string(), Arc::new(agent1)).await.unwrap();
    chain.add_agent("summarizer".to_string(), Arc::new(agent2)).await.unwrap();

    // 执行链式调用
    use lumosai_core::agent::types::RuntimeContext;
    use serde_json::json;
    let context = RuntimeContext::new();
    let input = json!("LumosAI is a powerful AI framework.");
    let result = chain.execute(input, &context).await;

    assert!(result.is_ok(), "Chain execution should succeed");
    println!("✅ Agent Chain test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 19: Agent Parallel 并行执行
#[tokio::test]
async fn test_agent_parallel_execution() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建多个 Agent
    let agent1 = ctx
        .create_test_agent("agent1", "Provide technical analysis.")
        .unwrap();

    let agent2 = ctx
        .create_test_agent("agent2", "Provide business analysis.")
        .unwrap();

    let agent3 = ctx
        .create_test_agent("agent3", "Provide user perspective.")
        .unwrap();

    // 创建并行执行器
    let parallel = AgentParallel::new(Arc::new(agent1))
        .parallel(Arc::new(agent2))
        .parallel(Arc::new(agent3));

    // 并行执行
    let start = std::time::Instant::now();
    let results = parallel.execute("Evaluate LumosAI framework").await;
    let duration = start.elapsed();

    assert!(results.is_ok(), "Parallel execution should succeed");
    let outputs = results.unwrap();
    assert_eq!(outputs.len(), 3, "Should get 3 responses");

    for output in &outputs {
        assert!(!output.is_empty(), "Response should not be empty");
    }

    // 并行执行应该比顺序快（理论上）
    println!(
        "✅ Agent Parallel test passed: 3 agents executed in {:?}",
        duration
    );

    ctx.teardown().await.unwrap();
}

/// 测试 20: Multi-Agent DAG 编排
#[tokio::test]
async fn test_agent_dag_orchestration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 DAG 编排器
    let orchestrator = lumosai_core::agent::AgentDagOrchestrator::new();

    // 创建 Agent
    let agent_a = ctx
        .create_test_agent("agent_a", "Process initial input.")
        .unwrap();

    let agent_b = ctx
        .create_test_agent("agent_b", "Process data from agent_a.")
        .unwrap();

    let agent_c = ctx
        .create_test_agent("agent_c", "Process data from agent_a.")
        .unwrap();

    let agent_d = ctx
        .create_test_agent("agent_d", "Combine results from b and c.")
        .unwrap();

    // 构建 DAG: A -> B -> D
    //           A -> C -> D
    orchestrator
        .add_root_agent("agent_a".to_string(), Arc::new(agent_a))
        .await
        .unwrap();

    orchestrator
        .add_agent("agent_b".to_string(), Arc::new(agent_b), vec![
            "agent_a".to_string(),
        ])
        .await
        .unwrap();

    orchestrator
        .add_agent("agent_c".to_string(), Arc::new(agent_c), vec![
            "agent_a".to_string(),
        ])
        .await
        .unwrap();

    orchestrator
        .add_agent("agent_d".to_string(), Arc::new(agent_d), vec![
            "agent_b".to_string(),
            "agent_c".to_string(),
        ])
        .await
        .unwrap();

    // 验证 DAG
    let validation = orchestrator.validate().await;
    assert!(validation.is_ok(), "DAG should be valid (no cycles)");

    // 执行 DAG
    let input = serde_json::json!({
        "message": "Test input for DAG orchestration"
    });

    let result = orchestrator
        .execute(input, &Default::default())
        .await;

    assert!(result.is_ok(), "DAG execution should succeed");

    println!("✅ Agent DAG orchestration test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 21: Agent 协作会话
#[tokio::test]
async fn test_agent_collaboration_session() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建多个 Agent
    let researcher = ctx
        .create_test_agent("researcher", "Research and gather information.")
        .unwrap();

    let writer = ctx
        .create_test_agent("writer", "Write content based on research.")
        .unwrap();

    // 简单的协作流程
    let research_result = researcher
        .generate_simple("Research about Rust programming")
        .await
        .unwrap();

    E2EAssertions::assert_non_empty_response(&research_result);

    // 将研究结果传递给写作 Agent
    let writing_prompt = format!("Write a summary based on: {}", research_result);
    let writing_result = writer.generate_simple(&writing_prompt).await.unwrap();

    E2EAssertions::assert_non_empty_response(&writing_result);

    println!("✅ Agent collaboration session test passed");

    ctx.teardown().await.unwrap();
}

// ===== 新增：2025 高级协作模式 E2E 测试 =====

/// 测试 22: Group Chat 协作模式
#[tokio::test]
async fn test_group_chat_collaboration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 Crew
    let crew = Crew::new("group_chat_crew".to_string(), CollaborationMode::GroupChat, 10);

    // 添加多个 Agent
    let agent1 = ctx.create_test_agent("researcher", "Research and provide facts.").unwrap();
    let agent2 = ctx.create_test_agent("analyst", "Analyze the facts.").unwrap();
    let agent3 = ctx.create_test_agent("writer", "Write a summary.").unwrap();

    let role1 = AgentRole::new("researcher".to_string(), "Research".to_string(), "Researcher".to_string());
    let role2 = AgentRole::new("analyst".to_string(), "Analyze".to_string(), "Analyst".to_string());
    let role3 = AgentRole::new("writer".to_string(), "Write".to_string(), "Writer".to_string());

    crew.add_agent("researcher".to_string(), Arc::new(agent1), role1).await.unwrap();
    crew.add_agent("analyst".to_string(), Arc::new(agent2), role2).await.unwrap();
    crew.add_agent("writer".to_string(), Arc::new(agent3), role3).await.unwrap();

    // 执行协作
    let results = crew.kickoff().await;
    assert!(results.is_ok(), "Group Chat collaboration should succeed");

    println!("✅ Group Chat collaboration test passed");
    ctx.teardown().await.unwrap();
}

/// 测试 23: Handoff 协作模式
#[tokio::test]
async fn test_handoff_collaboration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let crew = Crew::new("handoff_crew".to_string(), CollaborationMode::Handoff, 10);

    let agent1 = ctx.create_test_agent("agent1", "Handle initial request.").unwrap();
    let agent2 = ctx.create_test_agent("agent2", "Handle specialized request.").unwrap();

    let role1 = AgentRole::new("agent1".to_string(), "Initial".to_string(), "Initial handler".to_string());
    let role2 = AgentRole::new("agent2".to_string(), "Specialist".to_string(), "Specialist".to_string());

    crew.add_agent("agent1".to_string(), Arc::new(agent1), role1).await.unwrap();
    crew.add_agent("agent2".to_string(), Arc::new(agent2), role2).await.unwrap();

    let results = crew.kickoff().await;
    assert!(results.is_ok(), "Handoff collaboration should succeed");

    println!("✅ Handoff collaboration test passed");
    ctx.teardown().await.unwrap();
}

/// 测试 24: Reflection 协作模式
#[tokio::test]
async fn test_reflection_collaboration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let crew = Crew::new("reflection_crew".to_string(), CollaborationMode::Reflection, 10);

    let generator = ctx.create_test_agent("generator", "Generate content.").unwrap();
    let critic = ctx.create_test_agent("critic", "Critique and improve.").unwrap();

    let role1 = AgentRole::new("generator".to_string(), "Generate".to_string(), "Generator".to_string());
    let role2 = AgentRole::new("critic".to_string(), "Critique".to_string(), "Critic".to_string());

    crew.add_agent("generator".to_string(), Arc::new(generator), role1).await.unwrap();
    crew.add_agent("critic".to_string(), Arc::new(critic), role2).await.unwrap();

    let results = crew.kickoff().await;
    assert!(results.is_ok(), "Reflection collaboration should succeed");

    println!("✅ Reflection collaboration test passed");
    ctx.teardown().await.unwrap();
}

/// 测试 25: Magentic 协作模式
#[tokio::test]
async fn test_magentic_collaboration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let crew = Crew::new("magentic_crew".to_string(), CollaborationMode::Magentic, 10);

    let manager = ctx.create_test_agent("manager", "Plan and manage tasks.").unwrap();
    let worker1 = ctx.create_test_agent("worker1", "Execute tasks.").unwrap();
    let worker2 = ctx.create_test_agent("worker2", "Execute tasks.").unwrap();

    let role1 = AgentRole::new("manager".to_string(), "Manage".to_string(), "Manager".to_string());
    let role2 = AgentRole::new("worker1".to_string(), "Work".to_string(), "Worker 1".to_string());
    let role3 = AgentRole::new("worker2".to_string(), "Work".to_string(), "Worker 2".to_string());

    crew.add_agent("manager".to_string(), Arc::new(manager), role1).await.unwrap();
    crew.add_agent("worker1".to_string(), Arc::new(worker1), role2).await.unwrap();
    crew.add_agent("worker2".to_string(), Arc::new(worker2), role3).await.unwrap();

    let results = crew.kickoff().await;
    assert!(results.is_ok(), "Magentic collaboration should succeed");

    println!("✅ Magentic collaboration test passed");
    ctx.teardown().await.unwrap();
}

/// 测试 26: Debate 协作模式
#[tokio::test]
async fn test_debate_collaboration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let crew = Crew::new("debate_crew".to_string(), CollaborationMode::Debate, 10);

    let proposer = ctx.create_test_agent("proposer", "Argue for the proposal.").unwrap();
    let opposer = ctx.create_test_agent("opposer", "Argue against the proposal.").unwrap();
    let judge = ctx.create_test_agent("judge", "Judge the debate.").unwrap();

    let role1 = AgentRole::new("proposer".to_string(), "Propose".to_string(), "Proposer".to_string());
    let role2 = AgentRole::new("opposer".to_string(), "Oppose".to_string(), "Opposer".to_string());
    let role3 = AgentRole::new("judge".to_string(), "Judge".to_string(), "Judge".to_string());

    crew.add_agent("proposer".to_string(), Arc::new(proposer), role1).await.unwrap();
    crew.add_agent("opposer".to_string(), Arc::new(opposer), role2).await.unwrap();
    crew.add_agent("judge".to_string(), Arc::new(judge), role3).await.unwrap();

    let results = crew.kickoff().await;
    assert!(results.is_ok(), "Debate collaboration should succeed");

    println!("✅ Debate collaboration test passed");
    ctx.teardown().await.unwrap();
}

/// 测试 27: MakerChecker 协作模式
#[tokio::test]
async fn test_maker_checker_collaboration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let crew = Crew::new("maker_checker_crew".to_string(), CollaborationMode::MakerChecker, 10);

    let maker = ctx.create_test_agent("maker", "Create content.").unwrap();
    let checker = ctx.create_test_agent("checker", "Review and approve.").unwrap();

    let role1 = AgentRole::new("maker".to_string(), "Make".to_string(), "Maker".to_string());
    let role2 = AgentRole::new("checker".to_string(), "Check".to_string(), "Checker".to_string());

    crew.add_agent("maker".to_string(), Arc::new(maker), role1).await.unwrap();
    crew.add_agent("checker".to_string(), Arc::new(checker), role2).await.unwrap();

    let results = crew.kickoff().await;
    assert!(results.is_ok(), "MakerChecker collaboration should succeed");

    println!("✅ MakerChecker collaboration test passed");
    ctx.teardown().await.unwrap();
}

