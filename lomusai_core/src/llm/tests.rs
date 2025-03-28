#[cfg(test)]
mod tests {
    use crate::llm::{AnthropicProvider, LlmOptions, LlmProvider, Message, OpenAiProvider, Role};

    // 这些测试使用内联的测试数据，不依赖于外部HTTP模拟库
    
    // 测试LLM选项结构
    #[test]
    fn test_llm_options_default() {
        let options = LlmOptions::default();
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.max_tokens, Some(1000));
        assert_eq!(options.stream, false);
        assert!(options.stop.is_none());
        assert!(options.model.is_none());
        assert!(options.extra.is_empty());
    }
    
    // 测试消息结构
    #[test]
    fn test_message_creation() {
        let message = Message::new(Role::User, "Hello".to_string(), None, None);
        
        assert_eq!(message.role, Role::User);
        assert_eq!(message.content, "Hello");
        assert!(message.metadata.is_none());
        assert!(message.name.is_none());
    }
    
    // 测试Anthropic嵌入错误
    #[tokio::test]
    async fn test_anthropic_embedding_error() {
        // Anthropic没有嵌入API，所以这应该返回一个错误
        let provider = AnthropicProvider::new(
            "fake-api-key".to_string(),
            "claude-2".to_string(),
        );
        
        // 调用嵌入方法
        let result = provider.get_embedding("Hello").await;
        
        // 验证错误
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not provide an embedding API"));
    }
    
    // 以下测试需要实际的API密钥才能运行，所以默认被忽略
    // 要运行这些测试，需要设置环境变量OPENAI_API_KEY和ANTHROPIC_API_KEY
    
    #[tokio::test]
    #[ignore]
    async fn test_openai_integration() {
        // 只有在提供API密钥时才运行此测试
        let api_key = std::env::var("OPENAI_API_KEY").expect("需要设置OPENAI_API_KEY环境变量");
        
        let provider = OpenAiProvider::new(
            api_key,
            "gpt-3.5-turbo".to_string(),
        );
        
        // 创建请求选项
        let options = LlmOptions::default()
            .with_temperature(0.7)
            .with_max_tokens(50);
        
        // 调用生成方法
        let response = provider.generate("Say hello", &options).await;
        
        // 验证响应成功
        assert!(response.is_ok());
        println!("OpenAI response: {}", response.unwrap());
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_openai_embedding_integration() {
        // 只有在提供API密钥时才运行此测试
        let api_key = std::env::var("OPENAI_API_KEY").expect("需要设置OPENAI_API_KEY环境变量");
        
        let provider = OpenAiProvider::new(
            api_key,
            "gpt-3.5-turbo".to_string(),
        );
        
        // 调用嵌入方法
        let embedding = provider.get_embedding("Hello world").await;
        
        // 验证嵌入成功
        assert!(embedding.is_ok());
        let embedding_vec = embedding.unwrap();
        assert!(!embedding_vec.is_empty());
        println!("Embedding length: {}", embedding_vec.len());
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_anthropic_integration() {
        // 只有在提供API密钥时才运行此测试
        let api_key = std::env::var("ANTHROPIC_API_KEY").expect("需要设置ANTHROPIC_API_KEY环境变量");
        
        let provider = AnthropicProvider::new(
            api_key,
            "claude-3-opus-20240229".to_string(),
        );
        
        // 创建请求选项
        let options = LlmOptions::default()
            .with_temperature(0.7)
            .with_max_tokens(50);
            
        let messages = vec![
            Message::new(Role::User, "Say hello".to_string(), None, None),
        ];
        
        // 调用生成方法
        let response = provider.generate_with_messages(&messages, &options).await;
        
        // 验证响应成功
        assert!(response.is_ok());
        println!("Anthropic response: {}", response.unwrap());
    }
} 