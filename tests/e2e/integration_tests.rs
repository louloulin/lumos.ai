//! E2E 测试：集成场景

mod framework;
use framework::E2ETestContext;
use lumosai_core::agent::Agent;

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
        "You are a writer. Create content based on research."
    ).unwrap();

    // 研究阶段
    let research_result = researcher
        .generate_simple("Research about AI")
        .await
        .unwrap();
    assert!(!research_result.is_empty());

    // 写作阶段（使用研究结果）
    let writing_prompt = format!("Write based on this research: {}", research_result);
    let article = writer.generate_simple(&writing_prompt).await.unwrap();
    assert!(!article.is_empty());

    ctx.teardown().await.unwrap();
}

/// 测试 11: 端到端工作流
#[tokio::test]
async fn test_complete_workflow() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 1. 用户注册
    let user = ctx
        .register_test_user("workflow_user@test.com", "WorkflowPass123")
        .await
        .unwrap();

    // 2. 用户登录
    let token = ctx
        .login_test_user("workflow_user@test.com", "WorkflowPass123")
        .await
        .unwrap();

    // 3. 使用 Token 验证权限
    let validated_user = ctx.auth.validate_token(&token).await.unwrap();
    assert_eq!(validated_user.id, user.id);

    // 4. 创建 Agent（需要权限）
    let agent = ctx.create_test_agent(
        "user_agent",
        "You are a personalized assistant"
    ).unwrap();

    // 5. 使用 Agent
    let response = agent.generate_simple("Hello!").await.unwrap();
    assert!(!response.is_empty());

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
            agent_clone
                .generate_simple(&format!("Request {}", i))
                .await
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    let results = futures::future::join_all(handles).await;

    // 验证所有请求都成功
    let success_count = results.iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
        .count();

    assert!(success_count >= 4, "At least 4/5 requests should succeed");

    ctx.teardown().await.unwrap();
}

/// 测试 13: Agent Arc 共享
#[tokio::test]
async fn test_agent_arc_sharing() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = Arc::new(ctx.create_test_agent(
        "shared",
        "You are a shared agent"
    ).unwrap());

    // 通过 Arc 共享 Agent
    let agent1 = agent.clone();
    let agent2 = agent.clone();

    // 两个引用都应该可用
    let r1 = agent1.generate_simple("Test 1").await;
    let r2 = agent2.generate_simple("Test 2").await;

    assert!(r1.is_ok());
    assert!(r2.is_ok());

    ctx.teardown().await.unwrap();
}

/// 测试 14: 错误恢复
#[tokio::test]
async fn test_error_recovery() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx.create_test_agent(
        "resilient",
        "You are a resilient agent"
    ).unwrap();

    // 测试空输入处理
    let r1 = agent.generate_simple("").await;
    // 应该能处理（成功或有意义的错误）

    // 后续正常请求应该工作
    let r2 = agent.generate_simple("Normal request").await;
    assert!(r2.is_ok(), "Agent should recover from errors");

    ctx.teardown().await.unwrap();
}

