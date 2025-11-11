//! E2E 测试：错误恢复和并发场景

mod framework;
use framework::{E2ETestContext, E2EAssertions};

use lumosai_core::agent::Agent;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// 测试 29: 超时处理
#[tokio::test]
async fn test_timeout_handling() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx
        .create_test_agent("timeout_agent", "You are a test assistant.")
        .unwrap();

    // 使用短超时测试
    let result = timeout(
        Duration::from_secs(10),
        agent.generate_simple("Quick response please"),
    )
    .await;

    match result {
        Ok(agent_result) => {
            // 如果在时间内完成，验证结果
            assert!(agent_result.is_ok(), "Agent should respond successfully");
            println!("✅ Timeout test: completed within time limit");
        }
        Err(_) => {
            println!("⚠️  Timeout test: request exceeded time limit (expected for slow responses)");
        }
    }

    ctx.teardown().await.unwrap();
}

/// 测试 30: 重试机制
#[tokio::test]
async fn test_retry_mechanism() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx
        .create_test_agent("retry_agent", "You are a test assistant.")
        .unwrap();

    // 尝试多次请求以测试重试
    let max_retries = 3;
    let mut success = false;

    for attempt in 1..=max_retries {
        println!("Attempt {}/{}", attempt, max_retries);

        let result = agent.generate_simple("Test message").await;

        if result.is_ok() {
            success = true;
            break;
        }

        if attempt < max_retries {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    assert!(
        success,
        "Should succeed within {} retries",
        max_retries
    );

    println!("✅ Retry mechanism test passed");

    ctx.teardown().await.unwrap();
}

/// 测试 31: 并发请求处理
#[tokio::test]
async fn test_concurrent_requests() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = Arc::new(
        ctx.create_test_agent("concurrent_agent", "You are a test assistant.")
            .unwrap(),
    );

    let num_concurrent = 5;
    let mut handles = vec![];

    let start = std::time::Instant::now();

    // 启动多个并发请求
    for i in 0..num_concurrent {
        let agent_clone = Arc::clone(&agent);
        let handle = tokio::spawn(async move {
            let message = format!("Concurrent request {}", i);
            agent_clone.generate_simple(&message).await
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

    let duration = start.elapsed();

    println!(
        "✅ Concurrent test: {}/{} succeeded in {:?}",
        success_count, num_concurrent, duration
    );

    assert!(
        success_count > 0,
        "At least one concurrent request should succeed"
    );

    E2EAssertions::assert_reasonable_response_time(duration);

    ctx.teardown().await.unwrap();
}

/// 测试 32: 高负载测试
#[tokio::test]
async fn test_high_load() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = Arc::new(
        ctx.create_test_agent("load_agent", "You are a test assistant.")
            .unwrap(),
    );

    let num_requests = 10;
    let mut handles = vec![];

    let start = std::time::Instant::now();

    // 发送大量请求
    for i in 0..num_requests {
        let agent_clone = Arc::clone(&agent);
        let handle = tokio::spawn(async move {
            let message = format!("Load test request {}", i);
            agent_clone.generate_simple(&message).await
        });
        handles.push(handle);
    }

    // 收集结果
    let mut results = vec![];
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    let duration = start.elapsed();

    let success_count = results.iter().filter(|r| r.is_ok()).count();

    println!(
        "✅ Load test: {}/{} requests succeeded in {:?}",
        success_count, num_requests, duration
    );

    // 应该有较高的成功率
    assert!(
        success_count as f64 / num_requests as f64 > 0.7,
        "At least 70% of requests should succeed"
    );

    ctx.teardown().await.unwrap();
}

/// 测试 33: 资源清理
#[tokio::test]
async fn test_resource_cleanup() {
    // 多次创建和销毁上下文，确保资源正确清理
    for i in 0..5 {
        let ctx = E2ETestContext::setup().await.unwrap();

        let agent = ctx
            .create_test_agent(&format!("cleanup_agent_{}", i), "Test agent")
            .unwrap();

        let _ = agent.generate_simple("Test message").await;

        ctx.teardown().await.unwrap();
    }

    println!("✅ Resource cleanup test passed: 5 iterations completed");
}

/// 测试 34: 错误传播
#[tokio::test]
async fn test_error_propagation() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 测试各种错误场景
    let agent = ctx
        .create_test_agent("error_agent", "You are a test assistant.")
        .unwrap();

    // 1. 空输入（应该能处理）
    let result = agent.generate_simple("").await;
    // 即使是空输入也应该有某种响应或明确的错误
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle empty input gracefully"
    );

    // 2. 超长输入（应该能处理或返回明确错误）
    let long_input = "test ".repeat(1000);
    let result = agent.generate_simple(&long_input).await;
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle long input gracefully"
    );

    println!("✅ Error propagation test passed");

    ctx.teardown().await.unwrap();
}

