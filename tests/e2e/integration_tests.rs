//! E2E 测试：简化的集成场景（不依赖 auth）

#[path = "framework.rs"]
mod framework;
use framework::E2ETestContext;
use lumosai_core::agent::{Agent, AgentPipeline, AgentParallel};
use lumosai_core::agent::types::RuntimeContext;
use std::sync::Arc;
use serde_json::json;

/// 测试 10: Agent + Agent 协作（顺序执行）
#[tokio::test]
async fn test_multi_agent_collaboration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建研究 Agent
    let researcher = ctx.create_test_agent(
        "researcher",
        "You are a researcher. Gather information."
    ).unwrap();

    // 创建写作 Agent
    let writer = ctx.create_test_agent(
        "writer",
        "You are a writer. Create engaging content."
    ).unwrap();

    // Agent 1: 研究
    let research_result = researcher.generate_simple("Research about Rust").await.unwrap();
    assert!(!research_result.is_empty());

    // Agent 2: 基于研究结果写作
    let writing_prompt = format!("Write based on: {}", research_result);
    let writing_result = writer.generate_simple(&writing_prompt).await.unwrap();
    assert!(!writing_result.is_empty());

    println!("✅ Multi-agent collaboration test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 11: Agent Pipeline（管道式协作）
#[tokio::test]
async fn test_agent_pipeline() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 Agent
    let agent1 = Arc::new(ctx.create_test_agent(
        "analyzer",
        "You are an analyzer. Analyze the input."
    ).unwrap());

    let agent2 = Arc::new(ctx.create_test_agent(
        "summarizer",
        "You are a summarizer. Summarize the input."
    ).unwrap());

    // 创建管道
    let pipeline = AgentPipeline::new(agent1).pipe(agent2);

    // 执行管道
    let result = pipeline.execute("Analyze and summarize: Rust is a systems programming language").await;

    assert!(result.is_ok(), "Pipeline execution should succeed");
    let output = result.unwrap();
    assert!(!output.is_empty(), "Pipeline output should not be empty");

    println!("✅ Agent pipeline test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 14: Agent Parallel（并行执行）
#[tokio::test]
async fn test_agent_parallel_execution() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建多个 Agent
    let agent1 = Arc::new(ctx.create_test_agent(
        "technical",
        "You are a technical expert."
    ).unwrap());

    let agent2 = Arc::new(ctx.create_test_agent(
        "business",
        "You are a business expert."
    ).unwrap());

    let agent3 = Arc::new(ctx.create_test_agent(
        "user",
        "You are a user experience expert."
    ).unwrap());

    // 创建并行执行器
    let parallel = AgentParallel::new(agent1)
        .parallel(agent2)
        .parallel(agent3);

    // 并行执行
    let start = std::time::Instant::now();
    let results = parallel.execute("Evaluate Rust programming language").await;
    let duration = start.elapsed();

    assert!(results.is_ok(), "Parallel execution should succeed");
    let outputs = results.unwrap();
    assert_eq!(outputs.len(), 3, "Should get 3 responses");

    for output in &outputs {
        assert!(!output.is_empty(), "Each output should not be empty");
    }

    println!("✅ Agent parallel test passed: 3 agents executed in {:?}", duration);

    ctx.teardown().await.unwrap();
}

/// 测试 12: 并发请求处理
#[tokio::test]
async fn test_concurrent_requests() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = Arc::new(ctx.create_test_agent(
        "concurrent_agent",
        "You are a test assistant"
    ).unwrap());

    // 并发发送 5 个请求
    let mut handles = vec![];
    for i in 0..5 {
        let agent_clone = agent.clone();
        let handle: tokio::task::JoinHandle<Result<String, lumosai_core::Error>> = tokio::spawn(async move {
            agent_clone.generate_simple(&format!("Request {}", i)).await
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            success_count += 1;
        }
    }

    println!("✅ Concurrent test: {}/5 requests succeeded", success_count);
    assert!(success_count > 0, "At least one request should succeed");

    ctx.teardown().await.unwrap();
}

/// 测试 15: Agent DAG 编排
#[tokio::test]
async fn test_agent_dag_orchestration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 DAG 编排器
    let orchestrator = lumosai_core::agent::AgentDagOrchestrator::new();

    // 创建 Agent
    let agent_a = Arc::new(ctx.create_test_agent(
        "agent_a",
        "You are agent A. Process the input."
    ).unwrap());

    let agent_b = Arc::new(ctx.create_test_agent(
        "agent_b",
        "You are agent B. Refine the input."
    ).unwrap());

    let agent_c = Arc::new(ctx.create_test_agent(
        "agent_c",
        "You are agent C. Finalize the input."
    ).unwrap());

    // 构建 DAG: A -> B -> C
    orchestrator.add_agent("agent_a".to_string(), agent_a, vec![]).await.unwrap();
    orchestrator.add_agent("agent_b".to_string(), agent_b, vec!["agent_a".to_string()]).await.unwrap();
    orchestrator.add_agent("agent_c".to_string(), agent_c, vec!["agent_b".to_string()]).await.unwrap();

    // 执行 DAG
    let runtime_context = RuntimeContext::default();
    let input = json!({"message": "Process this through the DAG"});
    let results = orchestrator.execute(input, &runtime_context).await;

    assert!(results.is_ok(), "DAG execution should succeed");
    let outputs = results.unwrap();
    assert_eq!(outputs.len(), 3, "Should have 3 agent outputs");

    println!("✅ Agent DAG orchestration test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 13: 错误恢复
#[tokio::test]
async fn test_error_recovery() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx.create_test_agent(
        "resilient",
        "You are a resilient agent"
    ).unwrap();

    // 测试空输入处理
    let _r1 = agent.generate_simple("").await;
    // 应该能处理（成功或有意义的错误）

    // 后续正常请求应该工作
    let r2 = agent.generate_simple("Normal request").await;
    assert!(r2.is_ok(), "Agent should recover from errors");

    println!("✅ Error recovery test passed");

    ctx.teardown().await.unwrap();
}

