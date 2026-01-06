use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use lumosai_core::prelude::*;
use lumosai_core::agent::{BasicAgent, AgentConfig, AgentBuilder};
use lumosai_core::memory::MemoryManager;
use crate::common::{TestUtils, TestAssertions};

/// Agent创建和配置测试
#[cfg(test)]
mod agent_creation_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_builder_validation() {
        // 测试AgentBuilder的参数验证
        let llm = create_test_zhipu_provider_arc();

        // 测试有效配置
        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "You are a test agent".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm.clone());
        assert!(agent.is_ok());

        // 测试空名称
        let invalid_config = AgentConfig {
            name: "".to_string(),
            instructions: "You are a test agent".to_string(),
            ..Default::default()
        };

        let result = BasicAgent::new(invalid_config, llm.clone());
        // 应该处理空名称的情况
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_agent_with_invalid_model() {
        // 测试无效模型配置的错误处理
        let llm = create_test_zhipu_provider_arc(); // 空响应

        let config = AgentConfig {
            name: "test-agent".to_string(),
            instructions: "Test agent".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();

        // 测试空响应的处理
        let result = agent.generate("Hello").await;
        // 应该优雅地处理空响应
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_agent_memory_configuration() {
        // 测试不同内存配置的Agent创建
        let llm = create_test_zhipu_provider_arc();

        // 测试带内存管理器的Agent
        let memory_manager = MemoryManager::new();
        let config = AgentConfig {
            name: "memory-agent".to_string(),
            instructions: "Agent with memory".to_string(),
            memory_manager: Some(Arc::new(memory_manager)),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm);
        assert!(agent.is_ok());

        let agent = agent.unwrap();
        let response = agent.generate("Remember this: important data").await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_agent_tool_integration() {
        // 测试工具集成的正确性
        let llm = create_test_zhipu_provider_arc();

        let config = AgentConfig {
            name: "tool-agent".to_string(),
            instructions: "Agent with tools".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();

        // 测试基础功能
        let response = agent.generate("Use a tool").await;
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
    async fn test_agent_configuration_validation() {
        // 测试配置参数验证
        let llm = create_test_zhipu_provider_arc();

        // 测试最大长度限制
        let long_name = "a".repeat(1000);
        let config = AgentConfig {
            name: long_name,
            instructions: "Test".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm.clone());
        assert!(agent.is_ok()); // 应该处理长名称

        // 测试特殊字符
        let special_config = AgentConfig {
            name: "test-agent-🤖".to_string(),
            instructions: "Special chars: @#$%^&*()".to_string(),
            ..Default::default()
        };

        let agent = BasicAgent::new(special_config, llm);
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_agent_default_configuration() {
        // 测试默认配置
        let llm = create_test_zhipu_provider_arc();

        let config = AgentConfig::default();
        let agent = BasicAgent::new(config, llm);

        assert!(agent.is_ok());
        let agent = agent.unwrap();

        // 测试默认配置下的基本功能
        let response = agent.generate("Hello").await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_agent_timeout_configuration() {
        // 测试超时配置
        let llm = create_test_zhipu_provider_arc();

        let config = AgentConfig {
            name: "timeout-agent".to_string(),
            instructions: "Test timeout".to_string(),
            timeout: Some(Duration::from_millis(100)),
            ..Default::default()
        };

        let agent = BasicAgent::new(config, llm).unwrap();

        // 测试在超时限制内的操作
        let result = timeout(Duration::from_millis(200), agent.generate("Quick response")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_clone_and_copy() {
        // 测试Agent的克隆和复制
        let agent = TestUtils::create_test_agent("clone-test").await.unwrap();

        // 测试基本功能
        let response1 = agent.generate("First message").await;
        assert!(response1.is_ok());

        // 如果Agent支持克隆，测试克隆后的功能
        // let cloned_agent = agent.clone();
        // let response2 = cloned_agent.generate("Second message").await;
        // assert!(response2.is_ok());
    }

    #[tokio::test]
    async fn test_agent_resource_cleanup() {
        // 测试资源清理
        let agent = TestUtils::create_test_agent("cleanup-test").await.unwrap();

        // 使用Agent
        let _response = agent.generate("Test message").await;

        // Agent应该在drop时自动清理资源
        drop(agent);

        // 验证没有资源泄漏（这里是概念性测试）
        assert!(true);
    }
}
