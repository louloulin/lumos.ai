//! E2E 测试：简化的集成场景（不依赖 auth）

#[path = "framework.rs"]
mod framework;
use framework::E2ETestContext;
use lumosai_core::agent::Agent;
use std::sync::Arc;

/// 测试 10: Agent + Agent 协作
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

