//! 简单的Agent高级特性测试

#[cfg(test)]
mod tests {
    use super::super::*;
    use chrono::Utc;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_session_creation() {
        // 测试会话存储
        let storage = std::sync::Arc::new(MemorySessionStorage::new());
        let session_manager = SessionManager::new(storage);

        // 创建会话
        let session_id = "test_session".to_string();
        let agent_name = "test_agent".to_string();
        let user_id = Some("user_123".to_string());

        let session = session_manager
            .create_session(session_id.clone(), agent_name, user_id)
            .await
            .expect("Failed to create session");

        assert_eq!(session.metadata.session_id, session_id);
        assert_eq!(session.metadata.state, SessionState::Active);
        assert_eq!(session.messages.len(), 0);
    }

    #[tokio::test]
    async fn test_event_bus() {
        // 测试事件总线
        let event_bus = std::sync::Arc::new(EventBus::new(100));

        // 创建一个简单事件
        let event = AgentEvent::AgentStarted {
            agent_id: "test_agent".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        // 发布事件
        event_bus
            .publish(event)
            .await
            .expect("Failed to publish event");

        // 检查历史
        let history = event_bus.get_history(None).await;
        assert_eq!(history.len(), 1);
    }

    #[tokio::test]
    async fn test_orchestration_task_creation() {
        // 测试编排任务创建
        let task = CollaborationTask {
            id: "test_task".to_string(),
            name: "Test Task".to_string(),
            description: "A test collaboration task".to_string(),
            participants: vec!["agent1".to_string(), "agent2".to_string()],
            pattern: OrchestrationPattern::Sequential,
            input: json!({"message": "test"}),
            expected_output: None,
            timeout: Some(30),
            retry_config: None,
        };

        assert_eq!(task.participants.len(), 2);
        assert_eq!(task.id, "test_task");
    }

    #[tokio::test]
    async fn test_event_filter() {
        // 测试事件过滤器
        let filter = EventFilter::new()
            .with_event_types(vec!["AgentStarted".to_string()])
            .with_agent_ids(vec!["agent1".to_string()]);

        let event1 = AgentEvent::AgentStarted {
            agent_id: "agent1".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let event2 = AgentEvent::AgentStopped {
            agent_id: "agent1".to_string(),
            timestamp: Utc::now(),
            reason: "test".to_string(),
        };

        let event3 = AgentEvent::AgentStarted {
            agent_id: "agent2".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        assert!(filter.matches(&event1)); // 应该匹配
        assert!(!filter.matches(&event2)); // 事件类型不匹配
        assert!(!filter.matches(&event3)); // Agent ID不匹配
    }

    #[tokio::test]
    async fn test_session_state_transitions() {
        // 测试会话状态转换
        let storage = std::sync::Arc::new(MemorySessionStorage::new());
        let session_manager = SessionManager::new(storage.clone());

        // 创建会话
        let session_id = "state_test_session".to_string();
        let session = session_manager
            .create_session(session_id.clone(), "test_agent".to_string(), None)
            .await
            .expect("Failed to create session");

        assert_eq!(session.metadata.state, SessionState::Active);

        // 更新状态
        storage
            .update_session_state(&session_id, SessionState::Paused)
            .await
            .expect("Failed to update state");

        // 验证状态更新
        let updated_session = session_manager
            .get_session(&session_id)
            .await
            .expect("Failed to get session")
            .expect("Session not found");

        assert_eq!(updated_session.metadata.state, SessionState::Paused);
    }

    #[tokio::test]
    async fn test_tool_call_history() {
        // 测试工具调用历史
        let storage = std::sync::Arc::new(MemorySessionStorage::new());
        let session_manager = SessionManager::new(storage);

        // 创建会话
        let session_id = "tool_test_session".to_string();
        session_manager
            .create_session(session_id.clone(), "test_agent".to_string(), None)
            .await
            .expect("Failed to create session");

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

        // 验证工具调用记录
        let session = session_manager
            .get_session(&session_id)
            .await
            .expect("Failed to get session")
            .expect("Session not found");

        assert_eq!(session.tool_calls.len(), 1);
        assert_eq!(session.tool_calls[0].tool_name, "calculator");
        assert_eq!(session.tool_calls[0].status, ToolCallStatus::Success);
    }

    #[tokio::test]
    async fn test_metrics_event_handler() {
        // 测试指标事件处理器
        let metrics_handler = MetricsEventHandler::new();

        // 创建一些事件
        let events = vec![
            AgentEvent::AgentStarted {
                agent_id: "agent1".to_string(),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            },
            AgentEvent::MessageSent {
                from_agent: "agent1".to_string(),
                to_agent: Some("agent2".to_string()),
                message_id: "msg1".to_string(),
                content: "Hello".to_string(),
                timestamp: Utc::now(),
            },
            AgentEvent::ToolCalled {
                agent_id: "agent1".to_string(),
                tool_name: "calculator".to_string(),
                parameters: json!({}),
                timestamp: Utc::now(),
            },
        ];

        // 处理事件
        for event in events {
            metrics_handler
                .handle_event(&event)
                .await
                .expect("Failed to handle event");
        }

        // 检查指标
        let metrics = metrics_handler.get_metrics().await;
        assert_eq!(metrics.get("agent_started"), Some(&1));
        assert_eq!(metrics.get("message_sent"), Some(&1));
        assert_eq!(metrics.get("tool_called"), Some(&1));
        assert_eq!(metrics.get("total_events"), Some(&3));
    }

    #[tokio::test]
    async fn test_collaboration_session_creation() {
        // 测试协作会话创建
        let event_bus = std::sync::Arc::new(EventBus::new(100));
        
        let task = CollaborationTask {
            id: "collab_test".to_string(),
            name: "Collaboration Test".to_string(),
            description: "Test collaboration".to_string(),
            participants: vec!["agent1".to_string(), "agent2".to_string()],
            pattern: OrchestrationPattern::Sequential,
            input: json!({"message": "test"}),
            expected_output: None,
            timeout: Some(30),
            retry_config: None,
        };

        let agents = HashMap::new(); // 空的Agent映射用于测试

        let session = CollaborationSession::new(task, agents, event_bus);

        assert_eq!(session.task.participants.len(), 2);
        assert!(!session.id.is_empty());
        assert!(session.is_completed().await == false); // 初始状态应该是未完成
    }
}
