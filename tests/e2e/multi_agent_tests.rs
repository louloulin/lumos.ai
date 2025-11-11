//! E2E 测试：Multi-Agent 协作场景

mod framework;
use framework::{E2ETestContext, E2EAssertions};

use lumosai_core::agent::{Agent, AgentChain, AgentParallel};
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

    // 创建 Agent Chain
    let chain = AgentChain::new(Arc::new(agent1))
        .then(Arc::new(agent2));

    // 执行链式调用
    let result = chain.execute("LumosAI is a powerful AI framework.").await;

    assert!(result.is_ok(), "Chain execution should succeed");
    let output = result.unwrap();
    E2EAssertions::assert_non_empty_response(&output);

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
        E2EAssertions::assert_non_empty_response(output);
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

