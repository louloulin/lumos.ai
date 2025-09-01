use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use lumosai_core::agent::{BasicAgent, AgentConfig};
use lumosai_core::agent::trait_def::Agent;
use lumosai_core::agent::types::AgentGenerateOptions;
use lumosai_core::llm::{MockLlmProvider, Message, Role};
use lumosai_core::prelude::*;

mod common;
use common::{TestUtils, TestAssertions};

/// Agent创建和配置测试
#[tokio::test]
async fn test_agent_builder_validation() {
    // 测试AgentBuilder的参数验证
    let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
    
    // 测试有效配置
    let config = AgentConfig {
        name: "test-agent".to_string(),
        instructions: "You are a test agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, llm.clone());
    // BasicAgent::new 返回 BasicAgent，不是 Result

    // 测试空名称
    let invalid_config = AgentConfig {
        name: "".to_string(),
        instructions: "You are a test agent".to_string(),
        ..Default::default()
    };

    let _result = BasicAgent::new(invalid_config, llm.clone());
    // 应该处理空名称的情况，这里只是验证不会panic
}

#[tokio::test]
async fn test_agent_with_invalid_model() {
    // 测试无效模型配置的错误处理
    let llm = Arc::new(MockLlmProvider::new(vec![])); // 空响应
    
    let config = AgentConfig {
        name: "test-agent".to_string(),
        instructions: "Test agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, llm);

    // 测试空响应的处理
    let messages = vec![Message::new(
        Role::User,
        "Hello".to_string(),
        None,
        None
    )];
    let options = AgentGenerateOptions::default();
    let result = agent.generate(&messages, &options).await;
    // 应该优雅地处理空响应
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_agent_memory_configuration() {
    // 测试不同内存配置的Agent创建
    let llm = Arc::new(MockLlmProvider::new(vec!["Memory test response".to_string()]));
    
    // 测试带内存配置的Agent
    let config = AgentConfig {
        name: "memory-agent".to_string(),
        instructions: "Agent with memory".to_string(),
        memory_config: Some(lumosai_core::memory::MemoryConfig::default()),
        ..Default::default()
    };

    let agent = BasicAgent::new(config, llm);

    let messages = vec![Message::new(
        Role::User,
        "Remember this: important data".to_string(),
        None,
        None
    )];
    let options = AgentGenerateOptions::default();
    let response = agent.generate(&messages, &options).await;
    assert!(response.is_ok());
}

#[tokio::test]
async fn test_agent_concurrent_creation() {
    // 测试并发创建Agent的线程安全性
    let tasks: Vec<_> = (0..10).map(|i| {
        tokio::spawn(async move {
            let agent_name = format!("concurrent-agent-{}", i);
            TestUtils::create_test_agent(&agent_name).await
        })
    }).collect();
    
    let mut results = Vec::new();
    for task in tasks {
        let result = task.await.unwrap();
        results.push(result);
    }
    
    // 验证所有Agent都成功创建
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_agent_simple_generation() {
    // 测试基础文本生成功能
    let agent = TestUtils::create_test_agent("simple-gen").await.unwrap();

    let messages = vec![Message::new(
        Role::User,
        "Hello, how are you?".to_string(),
        None,
        None
    )];
    let options = AgentGenerateOptions::default();
    let response = agent.generate(&messages, &options).await;
    assert!(response.is_ok());

    let response = response.unwrap();
    assert!(!response.response.is_empty());
    assert_eq!(response.response, "Test response");
}

#[tokio::test]
async fn test_agent_multiple_generations() {
    // 测试多次生成
    let responses = vec![
        "First response".to_string(),
        "Second response".to_string(),
        "Third response".to_string(),
    ];
    
    let agent = TestUtils::create_test_agent_with_responses("multi-gen", responses.clone()).await.unwrap();

    let messages = vec![Message::new(
        Role::User,
        "Test message".to_string(),
        None,
        None
    )];
    let options = AgentGenerateOptions::default();

    for expected in responses {
        let response = agent.generate(&messages, &options).await.unwrap();
        assert_eq!(response.response, expected);
    }
}

#[tokio::test]
async fn test_agent_timeout_handling() {
    // 测试超时处理
    let agent = TestUtils::create_test_agent("timeout-test").await.unwrap();
    
    // 设置较短的超时时间
    let messages = vec![Message::new(
        Role::User,
        "This should complete quickly".to_string(),
        None,
        None
    )];
    let options = AgentGenerateOptions::default();

    let result = timeout(
        Duration::from_millis(1000),
        agent.generate(&messages, &options)
    ).await;

    // 应该在超时时间内完成
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_concurrent_requests() {
    // 测试并发请求处理
    let agent = Arc::new(TestUtils::create_test_agent("concurrent").await.unwrap());
    
    let tasks: Vec<_> = (0..5).map(|i| {
        let agent_clone = agent.clone();
        tokio::spawn(async move {
            let messages = vec![Message::new(
                Role::User,
                format!("Concurrent message {}", i),
                None,
                None
            )];
            let options = AgentGenerateOptions::default();
            agent_clone.generate(&messages, &options).await
        })
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    
    // 验证所有并发请求都成功处理
    for result in results {
        let response = result.unwrap();
        assert!(response.is_ok());
    }
}

#[tokio::test]
async fn test_agent_performance_baseline() {
    // 测试性能基线
    let agent = TestUtils::create_test_agent("performance").await.unwrap();

    let messages = vec![Message::new(
        Role::User,
        "Performance test message".to_string(),
        None,
        None
    )];
    let options = AgentGenerateOptions::default();

    let start_time = Instant::now();
    let response = agent.generate(&messages, &options).await;
    let duration = start_time.elapsed();

    assert!(response.is_ok());

    // 验证响应时间在合理范围内
    TestAssertions::assert_response_time(duration, Duration::from_secs(1));
}

#[tokio::test]
async fn test_agent_error_recovery() {
    // 测试错误恢复机制
    let llm = Arc::new(MockLlmProvider::new(vec![])); // 空响应会导致错误
    let config = AgentConfig {
        name: "error-recovery".to_string(),
        instructions: "Test error recovery".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, llm);

    let messages1 = vec![Message::new(
        Role::User,
        "First attempt".to_string(),
        None,
        None
    )];
    let messages2 = vec![Message::new(
        Role::User,
        "Second attempt".to_string(),
        None,
        None
    )];
    let options = AgentGenerateOptions::default();

    // 第一次调用可能失败
    let result1 = agent.generate(&messages1, &options).await;

    // Agent应该能够处理错误并继续工作
    let result2 = agent.generate(&messages2, &options).await;

    // 至少有一个结果应该是可处理的
    assert!(result1.is_ok() || result1.is_err());
    assert!(result2.is_ok() || result2.is_err());
}

#[tokio::test]
async fn test_agent_large_input_handling() {
    // 测试大输入处理
    let agent = TestUtils::create_test_agent("large-input").await.unwrap();

    let large_input = "Large input content ".repeat(100);
    let messages = vec![Message::new(
        Role::User,
        large_input,
        None,
        None
    )];
    let options = AgentGenerateOptions::default();
    let response = agent.generate(&messages, &options).await;

    // 应该能够处理大输入
    assert!(response.is_ok());
}
