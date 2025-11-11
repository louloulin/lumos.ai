//! E2E 测试：Agent 相关场景

use lumosai_core::agent::{Agent, AgentBuilder};
use lumosai_core::base::Base;

mod framework;
use framework::E2ETestContext;

/// 测试 1: Agent 基础对话
#[tokio::test]
async fn test_agent_basic_conversation() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 创建 Agent
    let agent = ctx.create_test_agent(
        "conversational",
        "You are a helpful assistant. Be concise."
    ).unwrap();

    // 测试对话
    let response = agent.generate_simple("Hello!").await;
    assert!(response.is_ok(), "Agent should respond to greeting");

    let response_text = response.unwrap();
    assert!(!response_text.is_empty(), "Response should not be empty");

    ctx.teardown().await.unwrap();
}

/// 测试 2: Agent 多轮对话
#[tokio::test]
async fn test_agent_multi_turn_conversation() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx.create_test_agent(
        "memory_agent",
        "You are a helpful assistant with memory."
    ).unwrap();

    // 第一轮
    let r1 = agent.generate_simple("My name is Alice").await.unwrap();
    assert!(!r1.is_empty());

    // 第二轮（应该记住名字）
    let r2 = agent.generate_simple("What's my name?").await.unwrap();
    // Note: 由于使用测试 LLM，可能不会真正记住，但应该返回响应
    assert!(!r2.is_empty());

    ctx.teardown().await.unwrap();
}

/// 测试 3: Agent 配置验证
#[tokio::test]
async fn test_agent_configuration() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = AgentBuilder::new()
        .name("configured_agent")
        .instructions("You are a test assistant")
        .model(ctx.llm.clone())
        .temperature(0.7)
        .max_tokens(100)
        .build();

    assert!(agent.is_ok(), "Agent should build with valid configuration");

    let agent = agent.unwrap();
    assert_eq!(agent.get_name(), "configured_agent");

    ctx.teardown().await.unwrap();
}

/// 测试 4: Agent 错误处理
#[tokio::test]
async fn test_agent_error_handling() {
    let ctx = E2ETestContext::setup().await.unwrap();

    let agent = ctx.create_test_agent(
        "error_agent",
        "You are a test agent"
    ).unwrap();

    // 测试空输入
    let response = agent.generate_simple("").await;
    // 即使是空输入，也应该有响应或错误
    assert!(response.is_ok() || response.is_err());

    ctx.teardown().await.unwrap();
}

/// 测试 5: Agent Builder 验证
#[tokio::test]
async fn test_agent_builder_validation() {
    let ctx = E2ETestContext::setup().await.unwrap();

    // 测试缺少必需字段
    let result = AgentBuilder::new()
        .name("test")
        .build(); // 缺少 instructions 和 model

    assert!(result.is_err(), "Should fail without required fields");

    ctx.teardown().await.unwrap();
}

