//! Convenience functions for creating LLM providers and common configurations
//! 
//! This module provides simplified APIs for creating LLM providers,
//! inspired by Mastra's model creation patterns.

use std::sync::Arc;
use crate::llm::{LlmProvider, OpenAiProvider, AnthropicProvider, QwenProvider};
use crate::Result;

/// Create an OpenAI provider with simplified configuration
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::openai;
/// 
/// let provider = openai("gpt-4").expect("Failed to create OpenAI provider");
/// ```
pub fn openai(model: &str) -> Result<Arc<dyn LlmProvider>> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| crate::Error::Configuration("OPENAI_API_KEY environment variable not set".to_string()))?;
    
    Ok(Arc::new(OpenAiProvider::new(api_key, model.to_string())))
}

/// Create an OpenAI provider with custom API key
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::openai_with_key;
/// 
/// let provider = openai_with_key("your-api-key", "gpt-4");
/// ```
pub fn openai_with_key(api_key: &str, model: &str) -> Arc<dyn LlmProvider> {
    Arc::new(OpenAiProvider::new(api_key.to_string(), model.to_string()))
}

/// Create an Anthropic provider with simplified configuration
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::anthropic;
/// 
/// let provider = anthropic("claude-3-sonnet").expect("Failed to create Anthropic provider");
/// ```
pub fn anthropic(model: &str) -> Result<Arc<dyn LlmProvider>> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| crate::Error::Configuration("ANTHROPIC_API_KEY environment variable not set".to_string()))?;
    
    Ok(Arc::new(AnthropicProvider::new(api_key, model.to_string())))
}

/// Create an Anthropic provider with custom API key
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::anthropic_with_key;
/// 
/// let provider = anthropic_with_key("your-api-key", "claude-3-sonnet");
/// ```
pub fn anthropic_with_key(api_key: &str, model: &str) -> Arc<dyn LlmProvider> {
    Arc::new(AnthropicProvider::new(api_key.to_string(), model.to_string()))
}

/// Create a DeepSeek provider with simplified configuration
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::deepseek;
/// 
/// let provider = deepseek("deepseek-chat").expect("Failed to create DeepSeek provider");
/// ```
pub fn deepseek(model: &str) -> Result<Arc<dyn LlmProvider>> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .map_err(|_| crate::Error::Configuration("DEEPSEEK_API_KEY environment variable not set".to_string()))?;
    
    Ok(Arc::new(QwenProvider::new(api_key, model.to_string(), "https://dashscope.aliyuncs.com/compatible-mode/v1")))
}

/// Create a DeepSeek provider with custom API key
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::deepseek_with_key;
/// 
/// let provider = deepseek_with_key("your-api-key", "deepseek-chat");
/// ```
pub fn deepseek_with_key(api_key: &str, model: &str) -> Arc<dyn LlmProvider> {
    Arc::new(QwenProvider::new(api_key.to_string(), model.to_string(), "https://dashscope.aliyuncs.com/compatible-mode/v1"))
}

/// Create a Qwen provider with simplified configuration
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::qwen;
/// 
/// let provider = qwen("qwen-turbo").expect("Failed to create Qwen provider");
/// ```
pub fn qwen(model: &str) -> Result<Arc<dyn LlmProvider>> {
    let api_key = std::env::var("QWEN_API_KEY")
        .map_err(|_| crate::Error::Configuration("QWEN_API_KEY environment variable not set".to_string()))?;
    
    Ok(Arc::new(QwenProvider::new(api_key, model.to_string(), "https://dashscope.aliyuncs.com/compatible-mode/v1")))
}

/// Create a Qwen provider with custom API key
/// 
/// # Example
/// 
/// ```rust
/// use lumosai_core::agent::convenience::qwen_with_key;
/// 
/// let provider = qwen_with_key("your-api-key", "qwen-turbo");
/// ```
pub fn qwen_with_key(api_key: &str, model: &str) -> Arc<dyn LlmProvider> {
    Arc::new(QwenProvider::new(api_key.to_string(), model.to_string(), "https://dashscope.aliyuncs.com/compatible-mode/v1"))
}

/// Model configuration builder for advanced settings
pub struct ModelBuilder {
    provider: Arc<dyn LlmProvider>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
}

impl ModelBuilder {
    /// Create a new model builder
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
            temperature: None,
            max_tokens: None,
            top_p: None,
        }
    }
    
    /// Set the temperature for the model
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
    
    /// Set the maximum tokens for the model
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// Set the top_p for the model
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
    
    /// Build the configured model
    pub fn build(self) -> Arc<dyn LlmProvider> {
        // For now, return the provider as-is
        // In the future, we could wrap it with configuration
        self.provider
    }
}

/// Extend LLM providers with configuration methods
pub trait LlmProviderExt {
    /// Configure the model with a builder pattern
    fn configure(self) -> ModelBuilder;
}

impl LlmProviderExt for Arc<dyn LlmProvider> {
    fn configure(self) -> ModelBuilder {
        ModelBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockLlmProvider;

    #[test]
    fn test_model_builder() {
        let mock_provider = Arc::new(MockLlmProvider::new(vec!["Hello!".to_string()]));
        
        let configured_model = mock_provider
            .configure()
            .temperature(0.7)
            .max_tokens(1000)
            .top_p(0.9)
            .build();
        
        // The configured model should be the same as the original for now
        assert!(Arc::ptr_eq(&configured_model, &mock_provider));
    }
    
    #[test]
    fn test_provider_creation_with_key() {
        let openai_provider = openai_with_key("test-key", "gpt-4");
        let anthropic_provider = anthropic_with_key("test-key", "claude-3-sonnet");
        let deepseek_provider = deepseek_with_key("test-key", "deepseek-chat");
        let qwen_provider = qwen_with_key("test-key", "qwen-turbo");
        
        // All providers should be created successfully
        assert!(!openai_provider.as_ref().name().is_empty());
        assert!(!anthropic_provider.as_ref().name().is_empty());
        assert!(!deepseek_provider.as_ref().name().is_empty());
        assert!(!qwen_provider.as_ref().name().is_empty());
    }
}
