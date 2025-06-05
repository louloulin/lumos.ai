//! Tests for the simplified Agent API
//! 
//! This module tests the new simplified API that provides Mastra-like
//! developer experience while maintaining Rust's performance advantages.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::llm::MockLlmProvider;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_agent_quick_api() {
        // Test the quick API for simple agent creation
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello from quick agent!".to_string()]));
        
        let agent = AgentFactory::quick("assistant", "You are a helpful assistant")
            .model(llm)
            .build()
            .expect("Failed to create agent with quick API");
        
        assert_eq!(agent.get_name(), "assistant");
        assert_eq!(agent.get_instructions(), "You are a helpful assistant");
        
        // Test that smart defaults are applied
        assert!(agent.get_tools().is_empty()); // No tools by default
    }

    #[tokio::test]
    async fn test_agent_builder_api() {
        // Test the builder API for more complex configuration
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello from builder agent!".to_string()]));
        
        let agent = AgentFactory::builder()
            .name("research_agent")
            .instructions("You are a research assistant")
            .model(llm)
            .max_tool_calls(5)
            .tool_timeout(60)
            .build()
            .expect("Failed to create agent with builder API");
        
        assert_eq!(agent.get_name(), "research_agent");
        assert_eq!(agent.get_instructions(), "You are a research assistant");
    }

    #[tokio::test]
    async fn test_web_agent_convenience() {
        // Test the web agent convenience function
        let llm = Arc::new(MockLlmProvider::new(vec!["I can help with web tasks!".to_string()]));
        
        let agent = AgentFactory::web_agent("web_helper", "You are a web-enabled assistant")
            .model(llm)
            .build()
            .expect("Failed to create web agent");
        
        assert_eq!(agent.get_name(), "web_helper");
        assert_eq!(agent.get_instructions(), "You are a web-enabled assistant");
        
        // Should have web tools
        let tools = agent.get_tools();
        assert!(tools.len() > 0);
        
        // Check for specific web tools
        assert!(agent.get_tool("http_request").is_some());
        assert!(agent.get_tool("web_scraper").is_some());
        assert!(agent.get_tool("json_api").is_some());
        assert!(agent.get_tool("url_validator").is_some());
    }

    #[tokio::test]
    async fn test_file_agent_convenience() {
        // Test the file agent convenience function
        let llm = Arc::new(MockLlmProvider::new(vec!["I can help with file operations!".to_string()]));
        
        let agent = AgentFactory::file_agent("file_helper", "You are a file management assistant")
            .model(llm)
            .build()
            .expect("Failed to create file agent");
        
        assert_eq!(agent.get_name(), "file_helper");
        assert_eq!(agent.get_instructions(), "You are a file management assistant");
        
        // Should have file tools
        let tools = agent.get_tools();
        assert!(tools.len() > 0);
        
        // Check for specific file tools
        assert!(agent.get_tool("file_read").is_some());
        assert!(agent.get_tool("file_write").is_some());
        assert!(agent.get_tool("directory_list").is_some());
        assert!(agent.get_tool("file_info").is_some());
    }

    #[tokio::test]
    async fn test_data_agent_convenience() {
        // Test the data agent convenience function
        let llm = Arc::new(MockLlmProvider::new(vec!["I can help with data processing!".to_string()]));
        
        let agent = AgentFactory::data_agent("data_helper", "You are a data processing assistant")
            .model(llm)
            .build()
            .expect("Failed to create data agent");
        
        assert_eq!(agent.get_name(), "data_helper");
        assert_eq!(agent.get_instructions(), "You are a data processing assistant");
        
        // Should have data tools
        let tools = agent.get_tools();
        assert!(tools.len() > 0);
        
        // Check for specific data tools
        assert!(agent.get_tool("json_processor").is_some());
        assert!(agent.get_tool("csv_parser").is_some());
        assert!(agent.get_tool("data_transformer").is_some());
    }

    #[tokio::test]
    async fn test_agent_builder_with_tool_collections() {
        // Test adding tool collections to a builder
        let llm = Arc::new(MockLlmProvider::new(vec!["I have many tools!".to_string()]));
        
        let agent = AgentFactory::builder()
            .name("multi_tool_agent")
            .instructions("You are a versatile assistant")
            .model(llm)
            .with_web_tools()
            .with_file_tools()
            .with_data_tools()
            .build()
            .expect("Failed to create multi-tool agent");
        
        assert_eq!(agent.get_name(), "multi_tool_agent");
        
        // Should have tools from all collections
        let tools = agent.get_tools();
        assert!(tools.len() >= 11); // At least 4 web + 4 file + 3 data tools
        
        // Check for tools from each collection
        assert!(agent.get_tool("http_request").is_some()); // Web tool
        assert!(agent.get_tool("file_read").is_some()); // File tool
        assert!(agent.get_tool("json_processor").is_some()); // Data tool
    }

    #[tokio::test]
    async fn test_smart_defaults() {
        // Test that smart defaults are applied correctly
        let llm = Arc::new(MockLlmProvider::new(vec!["Smart defaults work!".to_string()]));
        
        let agent = AgentFactory::quick("smart_agent", "You are a smart assistant")
            .model(llm)
            .build()
            .expect("Failed to create agent with smart defaults");
        
        // Smart defaults should be applied automatically
        assert_eq!(agent.get_name(), "smart_agent");
        assert_eq!(agent.get_instructions(), "You are a smart assistant");
        
        // The agent should be functional even with minimal configuration
        let messages = vec![crate::llm::Message {
            role: crate::llm::Role::User,
            content: "Hello!".to_string(),
            name: None,
            metadata: None,
        }];
        
        let options = AgentGenerateOptions::default();
        let result = agent.generate(&messages, &options).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_backward_compatibility() {
        // Test that the old API still works
        let llm = Arc::new(MockLlmProvider::new(vec!["Backward compatibility!".to_string()]));
        
        // Old way using AgentBuilder directly
        let old_agent = AgentBuilder::new()
            .name("old_style_agent")
            .instructions("You are an old-style agent")
            .model(llm.clone())
            .build()
            .expect("Failed to create agent with old API");
        
        // New way using AgentFactory::builder()
        let new_agent = AgentFactory::builder()
            .name("new_style_agent")
            .instructions("You are a new-style agent")
            .model(llm)
            .build()
            .expect("Failed to create agent with new API");
        
        // Both should work the same way
        assert_eq!(old_agent.get_name(), "old_style_agent");
        assert_eq!(new_agent.get_name(), "new_style_agent");
    }

    #[tokio::test]
    async fn test_error_handling() {
        // Test error handling for missing required fields
        
        // Missing name
        let result = AgentFactory::quick("", "You are an assistant")
            .build();
        assert!(result.is_err());
        
        // Missing instructions
        let llm = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        let result = AgentFactory::quick("agent", "")
            .model(llm.clone())
            .build();
        assert!(result.is_err());
        
        // Missing model
        let result = AgentFactory::quick("agent", "You are an assistant")
            .build();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_convenience_model_creation() {
        // Test convenience functions for model creation
        use crate::agent::convenience::*;
        
        // Test with mock environment variables
        std::env::set_var("OPENAI_API_KEY", "test-key");
        std::env::set_var("ANTHROPIC_API_KEY", "test-key");
        std::env::set_var("DEEPSEEK_API_KEY", "test-key");
        std::env::set_var("QWEN_API_KEY", "test-key");
        
        // These should work with environment variables
        let _openai_provider = openai("gpt-4").expect("Failed to create OpenAI provider");
        let _anthropic_provider = anthropic("claude-3-sonnet").expect("Failed to create Anthropic provider");
        let _deepseek_provider = deepseek("deepseek-chat").expect("Failed to create DeepSeek provider");
        let _qwen_provider = qwen("qwen-turbo").expect("Failed to create Qwen provider");
        
        // Test with explicit keys
        let _openai_with_key = openai_with_key("test-key", "gpt-4");
        let _anthropic_with_key = anthropic_with_key("test-key", "claude-3-sonnet");
        let _deepseek_with_key = deepseek_with_key("test-key", "deepseek-chat");
        let _qwen_with_key = qwen_with_key("test-key", "qwen-turbo");
        
        // Clean up environment variables
        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("ANTHROPIC_API_KEY");
        std::env::remove_var("DEEPSEEK_API_KEY");
        std::env::remove_var("QWEN_API_KEY");
    }
}
