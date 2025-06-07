//! Agent高级特性集成测试
//!
//! 测试会话持久化、多Agent编排和事件驱动架构的集成功能

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::llm::{LlmOptions, LlmProvider, Message, Role};
    use crate::error::Result;
    use std::sync::Arc;
    use async_trait::async_trait;
    use tokio::time::{sleep, Duration};
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    // Mock LLM Provider for testing
    struct MockLlmProvider {
        responses: Vec<String>,
        current_index: std::sync::atomic::AtomicUsize,
    }

    impl MockLlmProvider {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses,
                current_index: std::sync::atomic::AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl LlmProvider for MockLlmProvider {
        fn name(&self) -> &str {
            "mock"
        }

        async fn generate(&self, _prompt: &str, _options: &LlmOptions) -> Result<String> {
            let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self.responses.get(index % self.responses.len()).unwrap_or(&"Default response".to_string()).clone())
        }

        async fn generate_with_messages(&self, _messages: &[Message], _options: &LlmOptions) -> Result<String> {
            let index = self.current_index.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self.responses.get(index % self.responses.len()).unwrap_or(&"Default response".to_string()).clone())
        }

        async fn generate_stream<'a>(
            &'a self,
            _prompt: &'a str,
            _options: &'a LlmOptions,
        ) -> Result<futures::stream::BoxStream<'a, Result<String>>> {
            unimplemented!("Streaming not implemented for mock provider")
        }

        async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
            unimplemented!("Embeddings not implemented for mock provider")
        }
    }

    #[tokio::test]
    async fn test_session_persistence() {
        // 创建会话存储
        let storage = Arc::new(MemorySessionStorage::new());
        let session_manager = SessionManager::new(storage);

        // 创建会话
        let session_id = "test_session_001".to_string();
        let agent_name = "test_agent".to_string();
        let user_id = Some("user_123".to_string());

        let session = session_manager
            .create_session(session_id.clone(), agent_name, user_id)
            .await
            .expect("Failed to create session");

        assert_eq!(session.metadata.session_id, session_id);
        assert_eq!(session.metadata.state, SessionState::Active);
        assert_eq!(session.messages.len(), 0);

        // 添加消息
        let message = Message {
            role: Role::User,
            content: "Hello, agent!".to_string(),
            metadata: None,
            name: None,
        };

        session_manager
            .add_message(&session_id, message.clone())
            .await
            .expect("Failed to add message");

        // 验证消息已保存
        let updated_session = session_manager
            .get_session(&session_id)
            .await
            .expect("Failed to get session")
            .expect("Session not found");

        assert_eq!(updated_session.messages.len(), 1);
        assert_eq!(updated_session.messages[0].content, "Hello, agent!");
        assert_eq!(updated_session.metadata.message_count, 1);

        // 添加工具调用记录
        let tool_call = ToolCallHistory {
            call_id: "call_001".to_string(),
            tool_name: "calculator".to_string(),
            parameters: json!({"operation": "add", "a": 1, "b": 2}),
            result: Some(json!({"result": 3})),
            timestamp: Utc::now(),
            status: ToolCallStatus::Success,
            error: None,
        };

        session_manager
            .add_tool_call(&session_id, tool_call)
            .await
            .expect("Failed to add tool call");

        // 验证工具调用已保存
        let final_session = session_manager
            .get_session(&session_id)
            .await
            .expect("Failed to get session")
            .expect("Session not found");

        assert_eq!(final_session.tool_calls.len(), 1);
        assert_eq!(final_session.tool_calls[0].tool_name, "calculator");
    }

    #[tokio::test]
    async fn test_event_system() {
        // 创建事件总线
        let event_bus = Arc::new(EventBus::new(100));

        // 注册日志处理器
        let log_handler = Arc::new(LogEventHandler::new());
        event_bus
            .register_handler(log_handler)
            .await
            .expect("Failed to register log handler");

        // 注册指标处理器
        let metrics_handler = Arc::new(MetricsEventHandler::new());
        event_bus
            .register_handler(metrics_handler.clone())
            .await
            .expect("Failed to register metrics handler");

        // 发布一些事件
        let events = vec![
            AgentEvent::AgentStarted {
                agent_id: "agent_001".to_string(),
                timestamp: Utc::now(),
                metadata: std::collections::HashMap::new(),
            },
            AgentEvent::MessageSent {
                from_agent: "agent_001".to_string(),
                to_agent: Some("agent_002".to_string()),
                message_id: "msg_001".to_string(),
                content: "Hello!".to_string(),
                timestamp: Utc::now(),
            },
            AgentEvent::ToolCalled {
                agent_id: "agent_001".to_string(),
                tool_name: "calculator".to_string(),
                parameters: json!({"operation": "add"}),
                timestamp: Utc::now(),
            },
        ];

        for event in events {
            event_bus
                .publish(event)
                .await
                .expect("Failed to publish event");
        }

        // 等待事件处理
        sleep(Duration::from_millis(100)).await;

        // 检查指标
        let metrics = metrics_handler.get_metrics().await;
        assert_eq!(metrics.get("agent_started"), Some(&1));
        assert_eq!(metrics.get("message_sent"), Some(&1));
        assert_eq!(metrics.get("tool_called"), Some(&1));
        assert_eq!(metrics.get("total_events"), Some(&3));

        // 测试事件历史
        let history = event_bus.get_history(None).await;
        assert_eq!(history.len(), 3);

        // 测试事件过滤
        let filter = EventFilter::new()
            .with_event_types(vec!["AgentStarted".to_string()]);
        let filtered_history = event_bus.get_history(Some(filter)).await;
        assert_eq!(filtered_history.len(), 1);
    }

    #[tokio::test]
    async fn test_agent_orchestration() {
        // 创建事件总线
        let event_bus = Arc::new(EventBus::new(100));
        
        // 创建编排器
        let orchestrator = BasicOrchestrator::new(event_bus.clone());

        // 创建Mock Agent
        let mock_llm1 = Arc::new(MockLlmProvider::new(vec![
            "Agent 1 response".to_string(),
        ]));
        let mock_llm2 = Arc::new(MockLlmProvider::new(vec![
            "Agent 2 response".to_string(),
        ]));

        let agent1 = Arc::new(create_basic_agent(
            "agent_001",
            "You are agent 1",
            mock_llm1,
        )) as Arc<dyn Agent>;

        let agent2 = Arc::new(create_basic_agent(
            "agent_002", 
            "You are agent 2",
            mock_llm2,
        )) as Arc<dyn Agent>;

        // 创建协作任务
        let task = CollaborationTask {
            id: "task_001".to_string(),
            name: "Test Collaboration".to_string(),
            description: "Test multi-agent collaboration".to_string(),
            participants: vec!["agent_001".to_string(), "agent_002".to_string()],
            pattern: OrchestrationPattern::Sequential,
            input: json!({"message": "Hello from orchestrator"}),
            expected_output: None,
            timeout: Some(30),
            retry_config: None,
        };

        // 创建Agent映射
        let mut agents = std::collections::HashMap::new();
        agents.insert("agent_001".to_string(), agent1);
        agents.insert("agent_002".to_string(), agent2);

        // 创建协作会话
        let session_id = orchestrator
            .create_session(task, agents)
            .await
            .expect("Failed to create session");

        // 获取会话并执行
        if let Some(session_arc) = orchestrator.get_session(&session_id).await {
            let mut session = session_arc.lock().await;
            
            // 执行协作
            let result = orchestrator
                .execute_collaboration(&mut session)
                .await
                .expect("Failed to execute collaboration");

            // 验证结果
            assert!(result.is_object());
            let result_obj = result.as_object().unwrap();
            assert!(result_obj.contains_key("agent_001"));
            assert!(result_obj.contains_key("agent_002"));

            // 检查Agent状态
            let states = session.get_results().await;
            assert_eq!(states.len(), 2);
            
            for (agent_id, state) in states {
                match state {
                    AgentExecutionState::Completed(_) => {
                        println!("Agent {} completed successfully", agent_id);
                    }
                    other => {
                        panic!("Agent {} in unexpected state: {:?}", agent_id, other);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_parallel_orchestration() {
        // 创建事件总线
        let event_bus = Arc::new(EventBus::new(100));
        
        // 创建编排器
        let orchestrator = BasicOrchestrator::new(event_bus.clone());

        // 创建Mock Agent
        let mock_llm1 = Arc::new(MockLlmProvider::new(vec![
            "Parallel Agent 1 response".to_string(),
        ]));
        let mock_llm2 = Arc::new(MockLlmProvider::new(vec![
            "Parallel Agent 2 response".to_string(),
        ]));

        let agent1 = Arc::new(create_basic_agent(
            "parallel_agent_001",
            "You are parallel agent 1",
            mock_llm1,
        )) as Arc<dyn Agent>;

        let agent2 = Arc::new(create_basic_agent(
            "parallel_agent_002", 
            "You are parallel agent 2",
            mock_llm2,
        )) as Arc<dyn Agent>;

        // 创建并行协作任务
        let task = CollaborationTask {
            id: "parallel_task_001".to_string(),
            name: "Test Parallel Collaboration".to_string(),
            description: "Test parallel multi-agent collaboration".to_string(),
            participants: vec!["parallel_agent_001".to_string(), "parallel_agent_002".to_string()],
            pattern: OrchestrationPattern::Parallel,
            input: json!({"message": "Hello from parallel orchestrator"}),
            expected_output: None,
            timeout: Some(30),
            retry_config: None,
        };

        // 创建Agent映射
        let mut agents = std::collections::HashMap::new();
        agents.insert("parallel_agent_001".to_string(), agent1);
        agents.insert("parallel_agent_002".to_string(), agent2);

        // 创建协作会话
        let session_id = orchestrator
            .create_session(task, agents)
            .await
            .expect("Failed to create parallel session");

        // 获取会话并执行
        if let Some(session_arc) = orchestrator.get_session(&session_id).await {
            let mut session = session_arc.lock().await;
            
            // 记录开始时间
            let start_time = std::time::Instant::now();
            
            // 执行并行协作
            let result = orchestrator
                .execute_collaboration(&mut session)
                .await
                .expect("Failed to execute parallel collaboration");

            let duration = start_time.elapsed();
            println!("Parallel execution took: {:?}", duration);

            // 验证结果
            assert!(result.is_object());
            let result_obj = result.as_object().unwrap();
            assert!(result_obj.contains_key("parallel_agent_001"));
            assert!(result_obj.contains_key("parallel_agent_002"));

            // 检查Agent状态
            let states = session.get_results().await;
            assert_eq!(states.len(), 2);
            
            for (agent_id, state) in states {
                match state {
                    AgentExecutionState::Completed(_) => {
                        println!("Parallel Agent {} completed successfully", agent_id);
                    }
                    other => {
                        panic!("Parallel Agent {} in unexpected state: {:?}", agent_id, other);
                    }
                }
            }
        }
    }
}
