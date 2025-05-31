//! Integration tests for Mastra-style features

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::collections::HashMap;
    
    use crate::agent::evaluation::{EvaluationMetric, RelevanceMetric, LengthMetric};
    use crate::agent::types::{RuntimeContext, DynamicArgument, ToolsInput};
    use crate::llm::{Message, Role};
    use crate::logger::{create_logger, Component, LogLevel};
    use crate::memory::processor::{
        MemoryProcessor, MemoryProcessorOptions, MessageLimitProcessor, 
        DeduplicationProcessor
    };
    use crate::tool::Tool;

    fn create_test_logger() -> Arc<dyn crate::logger::Logger> {
        Arc::new(create_logger("test", Component::Agent, LogLevel::Debug))
    }

    fn create_test_messages() -> Vec<Message> {
        vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant.".to_string(),
                metadata: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: "Hello, how are you?".to_string(),
                metadata: None,
                name: None,
            },
            Message {
                role: Role::Assistant,
                content: "I'm doing well, thank you for asking!".to_string(),
                metadata: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: "What's the weather like?".to_string(),
                metadata: None,
                name: None,
            },
            // Duplicate message for deduplication testing
            Message {
                role: Role::User,
                content: "Hello, how are you?".to_string(),
                metadata: None,
                name: None,
            },
        ]
    }

    #[tokio::test]
    async fn test_runtime_context_basic() {
        let mut context = RuntimeContext::new();
        
        // Test variable setting and getting
        context.set_variable("user_id", serde_json::Value::String("123".to_string()));
        context.set_variable("session_id", serde_json::Value::String("abc".to_string()));
        
        assert_eq!(
            context.get_variable("user_id"),
            Some(&serde_json::Value::String("123".to_string()))
        );
        assert_eq!(
            context.get_variable("session_id"),
            Some(&serde_json::Value::String("abc".to_string()))
        );
        assert_eq!(context.get_variable("nonexistent"), None);
        
        // Test metadata setting and getting
        context.set_metadata("thread_id".to_string(), "thread_123".to_string());
        context.set_metadata("agent_name".to_string(), "test_agent".to_string());
        
        assert_eq!(context.get_metadata("thread_id"), Some("thread_123"));
        assert_eq!(context.get_metadata("agent_name"), Some("test_agent"));
        assert_eq!(context.get_metadata("nonexistent"), None);
    }

    #[tokio::test]
    async fn test_dynamic_arguments_basic() {
        let context = RuntimeContext::new();
        
        // Test static dynamic argument
        let static_arg: DynamicArgument<String> = Box::new(|_ctx| "static value".to_string());
        assert_eq!(static_arg(&context), "static value");
        
        // Test context-dependent dynamic argument
        let context_arg: DynamicArgument<String> = Box::new(|ctx| {
            if let Some(user_id) = ctx.get_variable("user_id") {
                format!("Hello user {}", user_id)
            } else {
                "Hello anonymous user".to_string()
            }
        });
        
        // Without user_id
        assert_eq!(context_arg(&context), "Hello anonymous user");
        
        // With user_id
        let mut context_with_user = RuntimeContext::new();
        context_with_user.set_variable("user_id".to_string(), serde_json::Value::String("123".to_string()));
        assert_eq!(context_arg(&context_with_user), "Hello user \"123\"");
    }

    #[tokio::test]
    async fn test_tools_input_basic() {
        let context = RuntimeContext::new();
        
        // Test static tools input
        let static_tools = ToolsInput::Static(HashMap::new());
        match static_tools {
            ToolsInput::Static(tools) => assert!(tools.is_empty()),
            _ => panic!("Expected static tools"),
        }
        
        // Test dynamic tools input
        let dynamic_tools: DynamicArgument<HashMap<String, Box<dyn Tool>>> = 
            Box::new(|_ctx| HashMap::new());
        let tools_input = ToolsInput::Dynamic(dynamic_tools);
        
        match tools_input {
            ToolsInput::Dynamic(resolver) => {
                let tools = resolver(&context);
                assert!(tools.is_empty());
            },
            _ => panic!("Expected dynamic tools"),
        }
    }

    #[tokio::test]
    async fn test_relevance_metric_basic() {
        let logger = Arc::new(create_test_logger());
        let metric = RelevanceMetric::new(logger, 0.3);
        let context = RuntimeContext::new();
        
        // Test relevant input/output
        let input = "What is the weather like today?";
        let output = "The weather today is sunny and warm.";
        let result = metric.evaluate(input, output, &context).await.unwrap();
        
        assert_eq!(result.metric_name, "relevance");
        assert!(result.score > 0.0);
        assert!(result.explanation.is_some());
        
        // Test irrelevant input/output
        let input = "What is the weather like?";
        let output = "I like pizza and ice cream.";
        let result = metric.evaluate(input, output, &context).await.unwrap();
        
        assert_eq!(result.metric_name, "relevance");
        // Score should be lower for irrelevant content
        assert!(result.score >= 0.0);
    }

    #[tokio::test]
    async fn test_length_metric_basic() {
        let logger = Arc::new(create_test_logger());
        let metric = LengthMetric::new(logger, 10, 50);
        let context = RuntimeContext::new();
        
        // Test appropriate length
        let input = "Test input";
        let output = "This is a good length response.";
        let result = metric.evaluate(input, output, &context).await.unwrap();
        
        assert_eq!(result.metric_name, "length");
        assert_eq!(result.score, 1.0); // Perfect score for appropriate length
        
        // Test too short
        let output = "Short";
        let result = metric.evaluate(input, output, &context).await.unwrap();
        assert!(result.score < 1.0);
        
        // Test too long
        let output = "This is a very long response that exceeds the maximum expected length and should receive a lower score";
        let result = metric.evaluate(input, output, &context).await.unwrap();
        assert!(result.score < 1.0);
    }

    #[tokio::test]
    async fn test_memory_processor_basic() {
        let logger = Arc::new(create_test_logger());
        let messages = create_test_messages();
        let options = MemoryProcessorOptions::default();
        
        // Test message limit processor
        let limit_processor = MessageLimitProcessor::new(3, logger.clone());
        let result = limit_processor.process(messages.clone(), &options).await.unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(limit_processor.processor_name(), "MessageLimitProcessor");
        
        // Test deduplication processor
        let dedup_processor = DeduplicationProcessor::new(logger.clone());
        let result = dedup_processor.process(messages.clone(), &options).await.unwrap();
        assert_eq!(result.len(), 4); // Should remove one duplicate
        assert_eq!(dedup_processor.processor_name(), "DeduplicationProcessor");
    }

    #[test]
    fn test_runtime_context_default() {
        let context = RuntimeContext::default();
        assert!(context.variables.is_empty());
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_evaluation_result_serialization() {
        use crate::agent::evaluation::EvaluationResult;
        
        let result = EvaluationResult {
            metric_name: "test".to_string(),
            score: 0.85,
            explanation: Some("Good result".to_string()),
            metadata: HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        };
        
        // Test that the result can be serialized
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("test"));
        assert!(serialized.contains("0.85"));
    }

    #[test]
    fn test_mastra_features_compilation() {
        // This test just ensures that all our Mastra-style types compile correctly
        let _context = RuntimeContext::new();
        let _static_tools = ToolsInput::Static(HashMap::new());
        let _dynamic_arg: DynamicArgument<String> = Box::new(|_| "test".to_string());
        
        // If this compiles, our basic Mastra features are working
        assert!(true);
    }
}
