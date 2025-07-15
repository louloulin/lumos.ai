use std::time::{SystemTime, UNIX_EPOCH, Duration};
use lumosai_core::llm::{LlmProvider, mock::MockLlmProvider, Message, Role};
use lumosai_core::agent::executor::BasicAgent;
use lumosai_core::agent::config::AgentConfig;
use lumosai_core::base::ComponentConfig;
use serde_json::{Value, json};
use tokio::time::timeout;

/// 测试错误处理修复 - 验证不再有panic风险
#[tokio::test]
async fn test_error_handling_fixes() {
    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LlmProvider>;
    
    let config = AgentConfig {
        name: "test_agent".to_string(),
        ..Default::default()
    };
    
    let component_config = ComponentConfig {
        name: "test_agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, component_config, llm);
    
    // 测试时间戳获取不会panic
    let messages = vec![Message {
        role: Role::User,
        content: "测试消息".to_string(),
        ..Default::default()
    }];
    
    // 这应该成功而不是panic
    let result = agent.generate(&messages, &Default::default()).await;
    assert!(result.is_ok(), "Agent生成应该成功，不应该panic");
}

/// 测试流式处理改进 - 验证智能分块
#[tokio::test]
async fn test_streaming_improvements() {
    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LlmProvider>;
    
    let config = AgentConfig {
        name: "streaming_test_agent".to_string(),
        ..Default::default()
    };
    
    let component_config = ComponentConfig {
        name: "streaming_test_agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, component_config, llm);
    
    let messages = vec![Message {
        role: Role::User,
        content: "请生成一个较长的响应来测试流式处理".to_string(),
        ..Default::default()
    }];
    
    // 测试流式处理
    let stream_result = agent.stream(&messages, &Default::default()).await;
    assert!(stream_result.is_ok(), "流式处理应该成功");
    
    // 验证流式响应
    if let Ok(mut stream) = stream_result {
        use futures::StreamExt;
        let mut chunks = Vec::new();
        
        // 收集前几个块
        for _ in 0..3 {
            if let Some(chunk_result) = stream.next().await {
                assert!(chunk_result.is_ok(), "流式块应该成功");
                if let Ok(chunk) = chunk_result {
                    chunks.push(chunk);
                }
            }
        }
        
        // 验证我们收到了多个块
        assert!(!chunks.is_empty(), "应该收到流式块");
        
        // 验证块不为空
        for chunk in &chunks {
            assert!(!chunk.is_empty(), "流式块不应为空");
        }
    }
}

/// 测试内存管理改进 - 验证优雅降级
#[tokio::test]
async fn test_memory_management_improvements() {
    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LlmProvider>;
    
    let config = AgentConfig {
        name: "memory_test_agent".to_string(),
        ..Default::default()
    };
    
    let component_config = ComponentConfig {
        name: "memory_test_agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, component_config, llm);
    
    // 测试未初始化内存的优雅处理
    let get_result = agent.get_memory_value("test_key").await;
    assert!(get_result.is_ok(), "获取内存值应该优雅处理未初始化情况");
    assert_eq!(get_result.unwrap(), None, "未初始化内存应该返回None");
    
    // 测试设置内存值的错误处理
    let set_result = agent.set_memory_value("test_key", json!("test_value")).await;
    assert!(set_result.is_err(), "设置内存值应该返回错误但不panic");
    
    // 验证错误消息有意义
    if let Err(error) = set_result {
        let error_msg = error.to_string();
        assert!(error_msg.contains("not initialized"), "错误消息应该说明内存未初始化");
        assert!(error_msg.contains("Please initialize"), "错误消息应该提供解决建议");
    }
}

/// 测试工具调用解析改进 - 验证健壮性
#[tokio::test]
async fn test_tool_parsing_improvements() {
    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LlmProvider>;
    
    let config = AgentConfig {
        name: "tool_test_agent".to_string(),
        ..Default::default()
    };
    
    let component_config = ComponentConfig {
        name: "tool_test_agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, component_config, llm);
    
    // 测试各种工具调用格式的解析
    let test_cases = vec![
        // JSON格式
        r#"```json
        {
            "tool": "calculator",
            "parameters": {"operation": "add", "a": 1, "b": 2}
        }
        ```"#,
        
        // 传统格式
        "Using the tool 'calculator' with parameters: {\"operation\": \"add\", \"a\": 1, \"b\": 2}",
        
        // 函数调用格式
        "calculator({\"operation\": \"add\", \"a\": 1, \"b\": 2})",
        
        // 无效格式（应该优雅处理）
        "这是一个没有工具调用的普通响应",
    ];
    
    for (i, test_case) in test_cases.iter().enumerate() {
        let parse_result = agent.parse_tool_calls(test_case);
        assert!(parse_result.is_ok(), "工具解析不应该失败，测试用例 {}", i);
        
        let tool_calls = parse_result.unwrap();
        if i < 3 {
            // 前三个应该解析出工具调用
            assert!(!tool_calls.is_empty(), "应该解析出工具调用，测试用例 {}", i);
            assert_eq!(tool_calls[0].name, "calculator", "工具名称应该正确，测试用例 {}", i);
        } else {
            // 最后一个应该没有工具调用
            assert!(tool_calls.is_empty(), "不应该解析出工具调用，测试用例 {}", i);
        }
    }
}

/// 测试并发安全改进 - 验证无死锁
#[tokio::test]
async fn test_concurrency_safety() {
    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LlmProvider>;
    
    let config = AgentConfig {
        name: "concurrent_test_agent".to_string(),
        ..Default::default()
    };
    
    let component_config = ComponentConfig {
        name: "concurrent_test_agent".to_string(),
        ..Default::default()
    };
    
    let agent = std::sync::Arc::new(BasicAgent::new(config, component_config, llm));
    
    let messages = vec![Message {
        role: Role::User,
        content: "并发测试消息".to_string(),
        ..Default::default()
    }];
    
    // 创建多个并发任务
    let mut handles = Vec::new();
    for i in 0..5 {
        let agent_clone = agent.clone();
        let messages_clone = messages.clone();
        
        let handle = tokio::spawn(async move {
            let result = agent_clone.generate(&messages_clone, &Default::default()).await;
            (i, result.is_ok())
        });
        
        handles.push(handle);
    }
    
    // 等待所有任务完成
    let mut success_count = 0;
    for handle in handles {
        let (task_id, success) = handle.await.expect("任务应该完成");
        if success {
            success_count += 1;
        }
        println!("任务 {} 完成，成功: {}", task_id, success);
    }
    
    // 验证大部分任务成功（允许一些失败，但不应该死锁）
    assert!(success_count >= 3, "至少应该有3个任务成功，实际成功: {}", success_count);
}

/// 测试超时处理 - 验证不会无限等待
#[tokio::test]
async fn test_timeout_handling() {
    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LlmProvider>;
    
    let config = AgentConfig {
        name: "timeout_test_agent".to_string(),
        ..Default::default()
    };
    
    let component_config = ComponentConfig {
        name: "timeout_test_agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, component_config, llm);
    
    let messages = vec![Message {
        role: Role::User,
        content: "超时测试消息".to_string(),
        ..Default::default()
    }];
    
    // 测试带超时的操作
    let timeout_result = timeout(
        Duration::from_secs(5),
        agent.generate(&messages, &Default::default())
    ).await;
    
    assert!(timeout_result.is_ok(), "操作应该在超时时间内完成");
    
    if let Ok(generate_result) = timeout_result {
        assert!(generate_result.is_ok(), "生成操作应该成功");
    }
}

/// 测试错误恢复 - 验证从错误状态恢复
#[tokio::test]
async fn test_error_recovery() {
    let llm = Box::new(MockLlmProvider::new()) as Box<dyn LlmProvider>;
    
    let config = AgentConfig {
        name: "recovery_test_agent".to_string(),
        ..Default::default()
    };
    
    let component_config = ComponentConfig {
        name: "recovery_test_agent".to_string(),
        ..Default::default()
    };
    
    let agent = BasicAgent::new(config, component_config, llm);
    
    let messages = vec![Message {
        role: Role::User,
        content: "恢复测试消息".to_string(),
        ..Default::default()
    }];
    
    // 执行多次操作，验证Agent能从错误中恢复
    for i in 0..3 {
        let result = agent.generate(&messages, &Default::default()).await;
        
        // 即使某次失败，后续操作仍应该能够执行
        println!("第 {} 次操作结果: {:?}", i + 1, result.is_ok());
        
        // 等待一小段时间
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    // 最后一次操作应该成功
    let final_result = agent.generate(&messages, &Default::default()).await;
    assert!(final_result.is_ok(), "最终操作应该成功，表明Agent已恢复");
}
