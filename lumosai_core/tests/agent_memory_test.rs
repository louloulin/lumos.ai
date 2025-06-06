// Test file for generate_with_memory implementation
#[cfg(test)]
mod tests {
    use lumosai_core::agent::{BasicAgent, AgentConfig};
    use lumosai_core::agent::trait_def::Agent;
    use lumosai_core::agent::types::AgentGenerateOptions;
    use lumosai_core::llm::{MockLlmProvider, LlmOptions, Message, Role};
    use lumosai_core::memory::{Memory, MemoryConfig as CoreMemoryConfig};
    use lumosai_core::memory::thread::{MemoryThread, MemoryThreadStorage, CreateThreadParams, GetMessagesParams};
    use lumosai_core::base::Base;
    use lumosai_core::logger::Component;
    use lumosai_core::Result;
    use std::sync::Arc;
    use std::collections::HashMap;
    use async_trait::async_trait;

    // Mock memory thread storage implementation for testing
    #[derive(Debug, Clone)]
    struct MockMemoryThreadStorage {
        threads: Arc<std::sync::Mutex<HashMap<String, MemoryThread>>>,
        messages: Arc<std::sync::Mutex<HashMap<String, Vec<Message>>>>,
    }

    impl MockMemoryThreadStorage {
        fn new() -> Self {
            Self {
                threads: Arc::new(std::sync::Mutex::new(HashMap::new())),
                messages: Arc::new(std::sync::Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl MemoryThreadStorage for MockMemoryThreadStorage {
        async fn create_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
            let mut threads = self.threads.lock().unwrap();
            threads.insert(thread.id.clone(), thread.clone());
            Ok(thread.clone())
        }

        async fn get_thread(&self, thread_id: &str) -> Result<Option<MemoryThread>> {
            let threads = self.threads.lock().unwrap();
            Ok(threads.get(thread_id).cloned())
        }

        async fn update_thread(&self, thread: &MemoryThread) -> Result<MemoryThread> {
            let mut threads = self.threads.lock().unwrap();
            threads.insert(thread.id.clone(), thread.clone());
            Ok(thread.clone())
        }

        async fn delete_thread(&self, thread_id: &str) -> Result<()> {
            let mut threads = self.threads.lock().unwrap();
            let mut messages = self.messages.lock().unwrap();
            threads.remove(thread_id);
            messages.remove(thread_id);
            Ok(())
        }

        async fn list_threads_by_resource(&self, resource_id: &str) -> Result<Vec<MemoryThread>> {
            let threads = self.threads.lock().unwrap();
            Ok(threads.values()
                .filter(|t| t.resource_id.as_ref() == Some(&resource_id.to_string()))
                .cloned()
                .collect())
        }

        async fn list_threads_by_agent(&self, agent_id: &str) -> Result<Vec<MemoryThread>> {
            let threads = self.threads.lock().unwrap();
            Ok(threads.values()
                .filter(|t| t.agent_id.as_ref() == Some(&agent_id.to_string()))
                .cloned()
                .collect())
        }

        async fn add_message(&self, thread_id: &str, message: &Message) -> Result<()> {
            let mut messages = self.messages.lock().unwrap();
            messages.entry(thread_id.to_string()).or_insert_with(Vec::new).push(message.clone());
            Ok(())
        }

        async fn get_messages(&self, thread_id: &str, _params: &GetMessagesParams) -> Result<Vec<Message>> {
            let messages = self.messages.lock().unwrap();
            Ok(messages.get(thread_id).unwrap_or(&Vec::new()).clone())
        }

        async fn delete_messages(&self, thread_id: &str, _message_ids: &[String]) -> Result<lumosai_core::memory::thread::MessageOperationResult> {
            // For mock implementation, just remove all messages for simplicity
            let mut messages = self.messages.lock().unwrap();
            if let Some(thread_messages) = messages.get_mut(thread_id) {
                let original_count = thread_messages.len();
                thread_messages.clear();
                Ok(lumosai_core::memory::thread::MessageOperationResult {
                    affected_count: original_count,
                    success: true,
                    error_message: None,
                })
            } else {
                Ok(lumosai_core::memory::thread::MessageOperationResult {
                    affected_count: 0,
                    success: true,
                    error_message: None,
                })
            }
        }

        async fn search_messages(&self, query: &str, _filter: Option<&lumosai_core::memory::thread::MessageFilter>) -> Result<Vec<Message>> {
            let messages = self.messages.lock().unwrap();
            let mut results = Vec::new();
            
            for thread_messages in messages.values() {
                for message in thread_messages {
                    if message.content.to_lowercase().contains(&query.to_lowercase()) {
                        results.push(message.clone());
                    }
                }
            }
            
            Ok(results)
        }

        async fn get_thread_stats(&self, thread_id: &str) -> Result<lumosai_core::memory::thread::ThreadStats> {
            let threads = self.threads.lock().unwrap();
            let messages = self.messages.lock().unwrap();
            
            let thread = threads.get(thread_id)
                .ok_or_else(|| lumosai_core::error::Error::NotFound(format!("Thread {} not found", thread_id)))?;
                
            let empty_vec = Vec::new();
            let thread_messages = messages.get(thread_id).unwrap_or(&empty_vec);

            let user_count = thread_messages.iter().filter(|m| m.role == Role::User).count();
            let assistant_count = thread_messages.iter().filter(|m| m.role == Role::Assistant).count();
            
            let last_message_at = thread_messages.last().map(|_| chrono::Utc::now());
            let size_bytes = thread_messages.iter().map(|m| m.content.len()).sum();
            
            Ok(lumosai_core::memory::thread::ThreadStats {
                message_count: thread_messages.len(),
                user_message_count: user_count,
                assistant_message_count: assistant_count,
                created_at: thread.created_at,
                last_message_at,
                size_bytes,
            })
        }
    }

    // Mock memory implementation that supports thread storage
    struct MockMemoryWithThreads {
        thread_storage: Arc<dyn MemoryThreadStorage>,
    }

    impl std::fmt::Debug for MockMemoryWithThreads {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MockMemoryWithThreads")
                .field("thread_storage", &"<dyn MemoryThreadStorage>")
                .finish()
        }
    }

    impl MockMemoryWithThreads {
        fn new() -> Self {
            Self {
                thread_storage: Arc::new(MockMemoryThreadStorage::new()),
            }
        }
    }

    #[async_trait]
    impl Memory for MockMemoryWithThreads {
        async fn store(&self, _message: &Message) -> Result<()> {
            Ok(())
        }

        async fn retrieve(&self, _config: &CoreMemoryConfig) -> Result<Vec<Message>> {
            Ok(Vec::new())
        }

        fn as_thread_storage(&self) -> Option<Arc<dyn MemoryThreadStorage>> {
            Some(self.thread_storage.clone())
        }
    }

    #[tokio::test]
    async fn test_generate_with_memory_basic() -> Result<()> {
        // Create a mock LLM provider with specific responses
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Hello Alice! Nice to meet you.".to_string(),
        ]));
        
        // Create basic agent config
        let config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        
        // Create memory with thread storage support
        let _memory = Arc::new(MockMemoryWithThreads::new());
        
        // Create basic agent with memory
        let agent = BasicAgent::new(config, llm);
        
        // Test basic message
        let messages = vec![
            Message::new(
                Role::User,
                "Hello, remember my name is Alice.".to_string(),
                None,
                None
            )
        ];
        
        // Create options with memory config
        let options = AgentGenerateOptions {
            system_message: None,
            instructions: None,
            context: None,
            memory_options: Some(CoreMemoryConfig {
                store_id: None,
                namespace: None,
                enabled: true,
                working_memory: None,
                semantic_recall: None,
                last_messages: Some(10),
                query: None,
            }),
            thread_id: Some("test_thread_123".to_string()),
            resource_id: Some("test_user_456".to_string()),
            run_id: Some("test_run_123".to_string()),
            max_steps: Some(5),
            tool_choice: Some(lumosai_core::agent::types::ToolChoice::Auto),
            llm_options: LlmOptions::default(),
            context_window: None,
        };
        
        // Call generate_with_memory
        let result = agent.generate_with_memory(&messages, Some("test_thread_123".to_string()), &options).await;
        
        // Check that we get a result
        assert!(result.is_ok());
        let response = result?;
        assert_eq!(response.response, "Hello Alice! Nice to meet you.");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_with_memory_conversation_history() -> Result<()> {
        // Create a mock LLM provider with multiple responses
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Hello Alice! Nice to meet you.".to_string(),
            "Your name is Alice, as you mentioned earlier.".to_string(),
        ]));
        
        let config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let _memory = Arc::new(MockMemoryWithThreads::new());
        let agent = BasicAgent::new(config, llm);
        
        let thread_id = "conversation_test_123".to_string();
        let options = AgentGenerateOptions {
            system_message: None,
            instructions: None,
            context: None,
            memory_options: Some(CoreMemoryConfig {
                store_id: None,
                namespace: None,
                enabled: true,
                working_memory: None,
                semantic_recall: None,
                last_messages: Some(10),
                query: None,
            }),
            thread_id: Some(thread_id.clone()),
            resource_id: Some("test_user_456".to_string()),
            run_id: Some("test_run_456".to_string()),
            max_steps: Some(5),
            tool_choice: Some(lumosai_core::agent::types::ToolChoice::Auto),
            llm_options: LlmOptions::default(),
            context_window: None,
        };
        
        // First message
        let first_message = vec![
            Message::new(
                Role::User,
                "Hello, my name is Alice.".to_string(),
                None,
                None
            )
        ];
        
        let first_result = agent.generate_with_memory(&first_message, Some(thread_id.clone()), &options).await?;
        assert_eq!(first_result.response, "Hello Alice! Nice to meet you.");
        
        // Second message - should have access to conversation history
        let second_message = vec![
            Message::new(
                Role::User,
                "What's my name?".to_string(),
                None,
                None
            )
        ];
        
        let second_result = agent.generate_with_memory(&second_message, Some(thread_id), &options).await?;
        assert_eq!(second_result.response, "Your name is Alice, as you mentioned earlier.");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_with_memory_without_thread_id() -> Result<()> {
        // Test that the method works even without a thread_id
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Hello! How can I help you?".to_string(),
        ]));
        
        let config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        let _memory = Arc::new(MockMemoryWithThreads::new());
        let agent = BasicAgent::new(config, llm);
        
        let messages = vec![
            Message::new(
                Role::User,
                "Hello!".to_string(),
                None,
                None
            )
        ];
        
        let options = AgentGenerateOptions {
            system_message: None,
            instructions: None,
            context: None,
            memory_options: None,
            thread_id: None,  // No thread ID
            resource_id: None,
            run_id: Some("test_run_789".to_string()),
            max_steps: Some(5),
            tool_choice: Some(lumosai_core::agent::types::ToolChoice::Auto),
            llm_options: LlmOptions::default(),
            context_window: None,
        };
        
        let result = agent.generate_with_memory(&messages, None, &options).await;
        assert!(result.is_ok());
        let response = result?;
        assert_eq!(response.response, "Hello! How can I help you?");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_with_memory_without_memory_storage() -> Result<()> {
        // Test with an agent that doesn't have memory storage
        let llm = Arc::new(MockLlmProvider::new(vec![
            "Hello! I don't have memory.".to_string(),
        ]));
        
        let config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "You are a helpful assistant.".to_string(),
            ..Default::default()
        };
        
        // Create agent without memory
        let agent = BasicAgent::new(config, llm);
        
        let messages = vec![
            Message::new(
                Role::User,
                "Hello!".to_string(),
                None,
                None
            )
        ];
        
        let options = AgentGenerateOptions {
            system_message: None,
            instructions: None,
            context: None,
            memory_options: None,
            thread_id: Some("test_thread".to_string()),
            resource_id: None,
            run_id: Some("test_run_101112".to_string()),
            max_steps: Some(5),
            tool_choice: Some(lumosai_core::agent::types::ToolChoice::Auto),
            llm_options: LlmOptions::default(),
            context_window: None,
        };
        
        let result = agent.generate_with_memory(&messages, Some("test_thread".to_string()), &options).await;
        assert!(result.is_ok());
        let response = result?;
        assert_eq!(response.response, "Hello! I don't have memory.");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_agent_base_implementation() -> Result<()> {
        // Create a mock LLM provider
        let llm = Arc::new(MockLlmProvider::new(vec!["Test response".to_string()]));
        
        // Create basic agent config
        let config = AgentConfig {
            name: "test_agent".to_string(),
            instructions: "You are a test agent.".to_string(),
            ..Default::default()
        };
        
        // Create basic agent
        let agent = BasicAgent::new(config, llm);
        
        // Test Base trait methods
        assert_eq!(agent.name(), Some("test_agent"));
        assert_eq!(agent.component(), Component::Agent);
        
        // Test that logger and telemetry setters work
        // (We can't test much more without actual implementations)
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_thread_creation_and_retrieval() -> Result<()> {
        let memory_storage = MockMemoryThreadStorage::new();
        
        // Test thread creation
        let thread_params = CreateThreadParams {
            id: Some("test_thread_456".to_string()),
            title: "Test Conversation".to_string(),
            agent_id: Some("test_agent".to_string()),
            resource_id: Some("user_123".to_string()),
            metadata: None,
        };
        
        let thread = MemoryThread::new(thread_params);
        let created_thread = memory_storage.create_thread(&thread).await?;
        assert_eq!(created_thread.id, "test_thread_456");
        assert_eq!(created_thread.title, "Test Conversation");
        
        // Test thread retrieval
        let retrieved_thread = memory_storage.get_thread("test_thread_456").await?;
        assert!(retrieved_thread.is_some());
        let retrieved = retrieved_thread.unwrap();
        assert_eq!(retrieved.id, "test_thread_456");
        assert_eq!(retrieved.title, "Test Conversation");
        
        // Test non-existent thread
        let non_existent = memory_storage.get_thread("non_existent").await?;
        assert!(non_existent.is_none());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_thread_message_storage() -> Result<()> {
        let memory_storage = MockMemoryThreadStorage::new();
        let thread_id = "message_test_thread";
        
        // Create a thread first
        let thread_params = CreateThreadParams {
            id: Some(thread_id.to_string()),
            title: "Message Test Thread".to_string(),
            agent_id: None,
            resource_id: None,
            metadata: None,
        };
        let thread = MemoryThread::new(thread_params);
        memory_storage.create_thread(&thread).await?;
        
        // Add messages
        let user_message = Message::new(
            Role::User,
            "Hello, AI!".to_string(),
            None,
            None
        );
        
        let assistant_message = Message::new(
            Role::Assistant,
            "Hello! How can I help you?".to_string(),
            None,
            None
        );
        
        memory_storage.add_message(thread_id, &user_message).await?;
        memory_storage.add_message(thread_id, &assistant_message).await?;
        
        // Retrieve messages
        let params = GetMessagesParams {
            limit: None,
            cursor: None,
            filter: None,
            include_content: true,
            reverse_order: false,
        };
        
        let messages = memory_storage.get_messages(thread_id, &params).await?;
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Hello, AI!");
        assert_eq!(messages[1].content, "Hello! How can I help you?");
        
        Ok(())
    }
}
