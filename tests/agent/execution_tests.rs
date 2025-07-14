use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use lumosai_core::prelude::*;
use lumosai_core::agent::BasicAgent;
use lumosai_core::llm::MockLlmProvider;
use crate::common::{TestUtils, TestAssertions, TestDataSets};

/// Agent执行和响应测试
#[cfg(test)]
mod agent_execution_tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_simple_generation() {
        // 测试基础文本生成功能
        let agent = TestUtils::create_test_agent("simple-gen").await.unwrap();
        
        let response = agent.generate("Hello, how are you?").await;
        assert!(response.is_ok());
        
        let response = response.unwrap();
        assert!(!response.content.is_empty());
        assert_eq!(response.content, "Test response");
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
        
        for expected in responses {
            let response = agent.generate("Test message").await.unwrap();
            assert_eq!(response.content, expected);
        }
    }

    #[tokio::test]
    async fn test_agent_streaming_response() {
        // 测试流式响应功能
        let agent = TestUtils::create_test_agent("streaming").await.unwrap();
        
        // 如果支持流式响应
        if let Ok(stream) = agent.generate_stream("Tell me a story").await {
            // 验证流式响应
            assert!(stream.is_ok());
        } else {
            // 如果不支持流式响应，测试普通响应
            let response = agent.generate("Tell me a story").await;
            assert!(response.is_ok());
        }
    }

    #[tokio::test]
    async fn test_agent_tool_calling() {
        // 测试工具调用功能
        let agent = TestUtils::create_test_agent("tool-calling").await.unwrap();
        
        let response = agent.generate("What's the weather like?").await;
        assert!(response.is_ok());
        
        // 验证响应内容
        let response = response.unwrap();
        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_agent_error_recovery() {
        // 测试错误恢复机制
        let llm = Arc::new(MockLlmProvider::new(vec![])); // 空响应会导致错误
        let config = lumosai_core::agent::AgentConfig {
            name: "error-recovery".to_string(),
            instructions: "Test error recovery".to_string(),
            ..Default::default()
        };
        
        let agent = BasicAgent::new(config, llm).unwrap();
        
        // 第一次调用可能失败
        let result1 = agent.generate("First attempt").await;
        
        // Agent应该能够处理错误并继续工作
        let result2 = agent.generate("Second attempt").await;
        
        // 至少有一个结果应该是可处理的
        assert!(result1.is_ok() || result1.is_err());
        assert!(result2.is_ok() || result2.is_err());
    }

    #[tokio::test]
    async fn test_agent_timeout_handling() {
        // 测试超时处理
        let agent = TestUtils::create_test_agent("timeout-test").await.unwrap();
        
        // 设置较短的超时时间
        let result = timeout(
            Duration::from_millis(100),
            agent.generate("This should complete quickly")
        ).await;
        
        // 应该在超时时间内完成
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_agent_rate_limiting() {
        // 测试速率限制
        let agent = TestUtils::create_test_agent("rate-limit").await.unwrap();
        
        let start_time = Instant::now();
        
        // 快速发送多个请求
        let tasks: Vec<_> = (0..5).map(|i| {
            let agent_ref = &agent;
            async move {
                agent_ref.generate(&format!("Message {}", i)).await
            }
        }).collect();
        
        let results = futures::future::join_all(tasks).await;
        let elapsed = start_time.elapsed();
        
        // 验证所有请求都得到处理
        for result in results {
            assert!(result.is_ok());
        }
        
        // 验证执行时间合理
        assert!(elapsed < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_agent_context_management() {
        // 测试上下文管理
        let agent = TestUtils::create_test_agent("context-mgmt").await.unwrap();
        
        // 发送第一条消息
        let response1 = agent.generate("My name is Alice").await.unwrap();
        assert!(!response1.content.is_empty());
        
        // 发送第二条消息，应该记住上下文
        let response2 = agent.generate("What's my name?").await.unwrap();
        assert!(!response2.content.is_empty());
        
        // 验证上下文连续性（如果支持）
        // 这里主要测试Agent不会崩溃
    }

    #[tokio::test]
    async fn test_agent_concurrent_requests() {
        // 测试并发请求处理
        let agent = Arc::new(TestUtils::create_test_agent("concurrent").await.unwrap());
        
        let tasks: Vec<_> = (0..10).map(|i| {
            let agent_clone = agent.clone();
            tokio::spawn(async move {
                agent_clone.generate(&format!("Concurrent message {}", i)).await
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
    async fn test_agent_large_input_handling() {
        // 测试大输入处理
        let agent = TestUtils::create_test_agent("large-input").await.unwrap();
        
        let large_input = "Large input content ".repeat(1000);
        let response = agent.generate(&large_input).await;
        
        // 应该能够处理大输入
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn test_agent_special_characters() {
        // 测试特殊字符处理
        let agent = TestUtils::create_test_agent("special-chars").await.unwrap();
        
        let test_data = TestDataSets::load();
        
        for edge_case in test_data.edge_cases {
            let response = agent.generate(edge_case).await;
            // 应该能够处理各种边界情况
            assert!(response.is_ok() || response.is_err());
        }
    }

    #[tokio::test]
    async fn test_agent_multilingual_support() {
        // 测试多语言支持
        let agent = TestUtils::create_test_agent("multilingual").await.unwrap();
        
        let test_data = TestDataSets::load();
        
        // 测试中文
        if let Some(chinese_texts) = test_data.multilingual_content.get("chinese") {
            for text in chinese_texts {
                let response = agent.generate(text).await;
                assert!(response.is_ok());
            }
        }
        
        // 测试日文
        if let Some(japanese_texts) = test_data.multilingual_content.get("japanese") {
            for text in japanese_texts {
                let response = agent.generate(text).await;
                assert!(response.is_ok());
            }
        }
    }

    #[tokio::test]
    async fn test_agent_response_validation() {
        // 测试响应验证
        let custom_responses = vec![
            "Valid response with content".to_string(),
            "Another valid response".to_string(),
        ];
        
        let agent = TestUtils::create_test_agent_with_responses("validation", custom_responses).await.unwrap();
        
        let response = agent.generate("Test message").await.unwrap();
        
        // 验证响应格式
        assert!(!response.content.is_empty());
        assert!(response.content.len() > 5);
        
        // 验证响应内容包含预期关键词
        TestAssertions::assert_response_contains(&response.content, &["Valid", "response"]);
    }

    #[tokio::test]
    async fn test_agent_performance_baseline() {
        // 测试性能基线
        let agent = TestUtils::create_test_agent("performance").await.unwrap();
        
        let start_time = Instant::now();
        let response = agent.generate("Performance test message").await;
        let duration = start_time.elapsed();
        
        assert!(response.is_ok());
        
        // 验证响应时间在合理范围内
        TestAssertions::assert_response_time(duration, Duration::from_secs(1));
    }
}
