//! 简化API测试
//! 
//! 测试Lumos简化API的基本功能

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    
    #[tokio::test]
    async fn test_vector_storage_creation() {
        // 测试内存存储创建
        let storage = crate::vector::memory().await;
        assert!(storage.is_ok());
        
        // 测试构建器模式
        let builder_storage = crate::vector::builder()
            .backend("memory")
            .batch_size(1000)
            .build()
            .await;
        assert!(builder_storage.is_ok());
    }
    
    #[tokio::test]
    async fn test_rag_system_creation() {
        // 创建存储
        let storage = crate::vector::memory().await
            .expect("Failed to create storage");
        
        // 测试简单RAG创建
        let rag_result = crate::rag::simple(storage.clone(), "openai").await;
        // 注意：这可能会失败，因为需要OpenAI API密钥
        // 我们只测试API调用不会panic
        let _ = rag_result;
        
        // 测试构建器模式
        let builder_rag = crate::rag::builder()
            .storage(storage)
            .embedding_provider("openai")
            .chunking_strategy("recursive")
            .chunk_size(800)
            .build()
            .await;
        // 同样，这可能会失败，但不应该panic
        let _ = builder_rag;
    }
    
    #[tokio::test]
    async fn test_agent_creation() {
        // 测试简单Agent创建
        let agent_result = crate::agent::simple("gpt-4", "You are helpful").await;
        // 注意：这可能会失败，因为需要OpenAI API密钥
        let _ = agent_result;
        
        // 测试构建器模式
        let builder_agent = crate::agent::builder()
            .name("TestAgent")
            .model("gpt-4")
            .system_prompt("You are a test assistant")
            .temperature(0.7)
            .build()
            .await;
        let _ = builder_agent;
    }
    
    #[tokio::test]
    async fn test_session_management() {
        // 测试会话创建
        let session = crate::session::create("test_agent", Some("user_123")).await;
        assert!(session.is_ok());
        
        if let Ok(session) = session {
            // 测试会话ID
            assert!(!session.id().is_empty());
            
            // 测试添加消息
            let message = Message {
                role: Role::User,
                content: "Hello!".to_string(),
                metadata: None,
                name: None,
            };
            
            let add_result = session.add_message(message).await;
            assert!(add_result.is_ok());
            
            // 测试获取消息
            let messages = session.get_messages().await;
            assert!(messages.is_ok());
            
            if let Ok(messages) = messages {
                assert_eq!(messages.len(), 1);
                assert_eq!(messages[0].content, "Hello!");
            }
        }
        
        // 测试构建器模式
        let builder_session = crate::session::builder()
            .agent_name("test_agent")
            .user_id("user_123")
            .title("Test Session")
            .build()
            .await;
        assert!(builder_session.is_ok());
    }
    
    #[tokio::test]
    async fn test_event_system() {
        // 测试事件总线创建
        let event_bus = crate::events::create_bus(100);
        assert!(!Arc::ptr_eq(&event_bus, &event_bus)); // 简单的存在性测试
        
        // 测试事件发布
        let publish_result = crate::events::publish(&event_bus, "agent_started", serde_json::json!({
            "agent_id": "test_agent"
        })).await;
        assert!(publish_result.is_ok());
        
        // 测试日志处理器注册
        let log_handler_result = crate::events::register_log_handler(&event_bus).await;
        assert!(log_handler_result.is_ok());
        
        // 测试指标处理器注册
        let metrics_handler_result = crate::events::register_metrics_handler(&event_bus).await;
        assert!(metrics_handler_result.is_ok());
        
        if let Ok(metrics_handler) = metrics_handler_result {
            // 等待事件处理
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            // 测试指标获取
            let metrics = metrics_handler.get_metrics().await;
            assert!(metrics.contains_key("agent_started"));
            assert!(metrics.contains_key("total_events"));
        }
        
        // 测试事件历史
        let history = crate::events::get_history(&event_bus, None).await;
        assert!(!history.is_empty());
        
        // 测试过滤器
        let filter = crate::events::filter()
            .agent_ids(vec!["test_agent".to_string()])
            .build();
        
        let filtered_history = crate::events::get_history(&event_bus, Some(filter)).await;
        assert!(!filtered_history.is_empty());
    }
    
    #[tokio::test]
    async fn test_orchestration_task_creation() {
        // 测试任务构建器
        let task = crate::orchestration::task()
            .name("Test Task")
            .description("A test collaboration task")
            .pattern(crate::orchestration::Pattern::Sequential)
            .input(serde_json::json!({"test": "data"}))
            .build();
        
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.description, "A test collaboration task");
        assert!(matches!(task.pattern, crate::orchestration::Pattern::Sequential));
        assert_eq!(task.agents.len(), 0); // 没有添加Agent
    }
    
    #[test]
    fn test_prelude_imports() {
        // 测试prelude模块是否正确导入了所有必要的类型
        
        // 测试错误类型
        let _error: Error = Error::Config("test".to_string());
        
        // 测试消息类型
        let _message = Message {
            role: Role::User,
            content: "test".to_string(),
            metadata: None,
            name: None,
        };
        
        // 测试UUID生成
        let _uuid = Uuid::new_v4();
        
        // 测试HashMap
        let _map: HashMap<String, String> = HashMap::new();
        
        // 测试Arc
        let _arc = Arc::new("test");
        
        // 测试时间
        let _now = Utc::now();
        
        // 测试JSON
        let _json = serde_json::json!({"test": "value"});
    }
    
    #[test]
    fn test_builder_patterns() {
        // 测试所有构建器模式的基本功能
        
        // 向量存储构建器
        let _vector_builder = crate::vector::builder()
            .backend("memory")
            .batch_size(1000);
        
        // RAG构建器
        let _rag_builder = crate::rag::builder()
            .embedding_provider("openai")
            .chunking_strategy("recursive")
            .chunk_size(800);
        
        // Agent构建器
        let _agent_builder = crate::agent::builder()
            .name("TestAgent")
            .model("gpt-4")
            .temperature(0.7);
        
        // 会话构建器
        let _session_builder = crate::session::builder()
            .agent_name("test_agent")
            .user_id("user_123");
        
        // 任务构建器
        let _task_builder = crate::orchestration::task()
            .name("Test Task")
            .description("Test");
        
        // 事件过滤器构建器
        let _filter_builder = crate::events::filter()
            .agent_ids(vec!["agent1".to_string()]);
    }
    
    #[test]
    fn test_serialization() {
        // 测试关键类型的序列化/反序列化
        
        // 测试AgentResponse
        let response = crate::agent::AgentResponse {
            content: "Hello".to_string(),
            metadata: None,
        };
        
        let json = serde_json::to_string(&response).expect("Failed to serialize");
        let _deserialized: crate::agent::AgentResponse = serde_json::from_str(&json)
            .expect("Failed to deserialize");
        
        // 测试OrchestrationResult
        let result = crate::orchestration::OrchestrationResult {
            task_id: "test".to_string(),
            results: HashMap::new(),
            execution_time_ms: 1000,
            status: "completed".to_string(),
        };
        
        let json = serde_json::to_string(&result).expect("Failed to serialize");
        let _deserialized: crate::orchestration::OrchestrationResult = serde_json::from_str(&json)
            .expect("Failed to deserialize");
    }
    
    #[test]
    fn test_framework_info() {
        // 测试框架信息
        assert_eq!(crate::FRAMEWORK_INFO, "Lumos - 企业级AI应用开发框架");
        assert!(!crate::VERSION.is_empty());
    }
}
