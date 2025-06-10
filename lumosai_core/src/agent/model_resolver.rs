//! Model name resolver for automatic LLM provider creation
//! 
//! This module provides automatic model name resolution to LLM providers,
//! enabling chain calls like `.model("gpt-4")` instead of manually creating providers.

use std::collections::HashMap;
use std::sync::Arc;
use crate::llm::{LlmProvider, OpenAiProvider, AnthropicProvider, QwenProvider};
use crate::{Error, Result};

/// Model resolver for automatic LLM provider creation
pub struct ModelResolver {
    /// API keys for different providers
    api_keys: HashMap<String, String>,
    /// Default model mappings
    model_mappings: HashMap<String, ProviderInfo>,
}

/// Provider information
#[derive(Debug, Clone)]
struct ProviderInfo {
    provider: String,
    model_name: String,
    requires_api_key: bool,
}

impl ModelResolver {
    /// Create a new model resolver
    pub fn new() -> Self {
        let mut resolver = Self {
            api_keys: HashMap::new(),
            model_mappings: HashMap::new(),
        };
        
        resolver.init_default_mappings();
        resolver.load_api_keys_from_env();
        resolver
    }
    
    /// Initialize default model mappings
    fn init_default_mappings(&mut self) {
        // OpenAI models
        self.add_mapping("gpt-4", "openai", "gpt-4", true);
        self.add_mapping("gpt-4-turbo", "openai", "gpt-4-turbo", true);
        self.add_mapping("gpt-3.5-turbo", "openai", "gpt-3.5-turbo", true);
        self.add_mapping("gpt-4o", "openai", "gpt-4o", true);
        self.add_mapping("gpt-4o-mini", "openai", "gpt-4o-mini", true);
        
        // Anthropic models
        self.add_mapping("claude-3-opus", "anthropic", "claude-3-opus-20240229", true);
        self.add_mapping("claude-3-sonnet", "anthropic", "claude-3-5-sonnet-20241022", true);
        self.add_mapping("claude-3-haiku", "anthropic", "claude-3-haiku-20240307", true);
        self.add_mapping("claude-3.5-sonnet", "anthropic", "claude-3-5-sonnet-20241022", true);
        
        // DeepSeek models (using QwenProvider with DeepSeek endpoint)
        self.add_mapping("deepseek-chat", "deepseek", "deepseek-chat", true);
        self.add_mapping("deepseek-coder", "deepseek", "deepseek-coder", true);
        self.add_mapping("deepseek-math", "deepseek", "deepseek-math", true);
        
        // Qwen models
        self.add_mapping("qwen-turbo", "qwen", "qwen-turbo", true);
        self.add_mapping("qwen-plus", "qwen", "qwen-plus", true);
        self.add_mapping("qwen-max", "qwen", "qwen-max", true);
        
        // Local models (Ollama)
        self.add_mapping("llama2", "ollama", "llama2", false);
        self.add_mapping("llama3", "ollama", "llama3", false);
        self.add_mapping("codellama", "ollama", "codellama", false);
        self.add_mapping("mistral", "ollama", "mistral", false);
    }
    
    /// Add a model mapping
    fn add_mapping(&mut self, alias: &str, provider: &str, model_name: &str, requires_api_key: bool) {
        self.model_mappings.insert(
            alias.to_string(),
            ProviderInfo {
                provider: provider.to_string(),
                model_name: model_name.to_string(),
                requires_api_key,
            }
        );
    }
    
    /// Load API keys from environment variables
    fn load_api_keys_from_env(&mut self) {
        // OpenAI
        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            self.api_keys.insert("openai".to_string(), key);
        }
        
        // Anthropic
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            self.api_keys.insert("anthropic".to_string(), key);
        }
        
        // DeepSeek
        if let Ok(key) = std::env::var("DEEPSEEK_API_KEY") {
            self.api_keys.insert("deepseek".to_string(), key);
        }
        
        // Qwen
        if let Ok(key) = std::env::var("DASHSCOPE_API_KEY") {
            self.api_keys.insert("qwen".to_string(), key);
        }
    }
    
    /// Resolve model name to LLM provider
    pub async fn resolve(&self, model_spec: &str) -> Result<Arc<dyn LlmProvider>> {
        // Parse model specification
        let (provider, model_name) = self.parse_model_spec(model_spec)?;
        
        // Get provider info
        let provider_info = if let Some(info) = self.model_mappings.get(&model_name) {
            info.clone()
        } else {
            // If not in mappings, try to infer from provider prefix
            ProviderInfo {
                provider: provider.clone(),
                model_name: model_name.clone(),
                requires_api_key: true,
            }
        };
        
        // Create provider
        self.create_provider(&provider_info).await
    }
    
    /// Parse model specification (e.g., "openai/gpt-4" or "gpt-4")
    fn parse_model_spec(&self, model_spec: &str) -> Result<(String, String)> {
        if let Some((provider, model)) = model_spec.split_once('/') {
            // Explicit provider specification: "openai/gpt-4"
            Ok((provider.to_string(), model.to_string()))
        } else {
            // Model name only: "gpt-4" - use mapping or infer
            if let Some(info) = self.model_mappings.get(model_spec) {
                Ok((info.provider.clone(), model_spec.to_string()))
            } else {
                // Try to infer provider from model name
                let provider = self.infer_provider(model_spec)?;
                Ok((provider, model_spec.to_string()))
            }
        }
    }
    
    /// Infer provider from model name
    fn infer_provider(&self, model_name: &str) -> Result<String> {
        if model_name.starts_with("gpt-") {
            Ok("openai".to_string())
        } else if model_name.starts_with("claude-") {
            Ok("anthropic".to_string())
        } else if model_name.starts_with("deepseek-") {
            Ok("deepseek".to_string())
        } else if model_name.starts_with("qwen-") {
            Ok("qwen".to_string())
        } else {
            Err(Error::Configuration(format!(
                "Cannot infer provider for model: {}. Use explicit format like 'openai/{}'",
                model_name, model_name
            )))
        }
    }
    
    /// Create LLM provider
    async fn create_provider(&self, info: &ProviderInfo) -> Result<Arc<dyn LlmProvider>> {
        match info.provider.as_str() {
            "openai" => {
                let api_key = self.get_api_key("openai")?;
                Ok(Arc::new(OpenAiProvider::new(api_key, info.model_name.clone())))
            },
            "anthropic" => {
                let api_key = self.get_api_key("anthropic")?;
                Ok(Arc::new(AnthropicProvider::new(api_key, info.model_name.clone())))
            },
            "deepseek" => {
                let api_key = self.get_api_key("deepseek")?;
                Ok(Arc::new(QwenProvider::new(
                    api_key,
                    info.model_name.clone(),
                    "https://api.deepseek.com/v1"
                )))
            },
            "qwen" => {
                let api_key = self.get_api_key("qwen")?;
                Ok(Arc::new(QwenProvider::new(
                    api_key,
                    info.model_name.clone(),
                    "https://dashscope.aliyuncs.com/compatible-mode/v1"
                )))
            },
            "ollama" => {
                // Ollama doesn't require API key
                // Note: This would need OllamaProvider implementation
                Err(Error::Configuration(
                    "Ollama provider not yet implemented".to_string()
                ))
            },
            _ => Err(Error::Configuration(format!(
                "Unsupported provider: {}", info.provider
            )))
        }
    }
    
    /// Get API key for provider
    fn get_api_key(&self, provider: &str) -> Result<String> {
        self.api_keys.get(provider)
            .cloned()
            .ok_or_else(|| Error::Configuration(format!(
                "API key not found for provider '{}'. Set environment variable: {}",
                provider,
                match provider {
                    "openai" => "OPENAI_API_KEY",
                    "anthropic" => "ANTHROPIC_API_KEY", 
                    "deepseek" => "DEEPSEEK_API_KEY",
                    "qwen" => "DASHSCOPE_API_KEY",
                    _ => "UNKNOWN_API_KEY"
                }
            )))
    }
    
    /// Add custom API key
    pub fn add_api_key(&mut self, provider: &str, api_key: &str) {
        self.api_keys.insert(provider.to_string(), api_key.to_string());
    }
    
    /// List supported models
    pub fn list_models(&self) -> Vec<String> {
        self.model_mappings.keys().cloned().collect()
    }
}

impl Default for ModelResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_model_spec() {
        let resolver = ModelResolver::new();
        
        // Test explicit provider
        let (provider, model) = resolver.parse_model_spec("openai/gpt-4").unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
        
        // Test model name only
        let (provider, model) = resolver.parse_model_spec("gpt-4").unwrap();
        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
    }
    
    #[test]
    fn test_infer_provider() {
        let resolver = ModelResolver::new();
        
        assert_eq!(resolver.infer_provider("gpt-4").unwrap(), "openai");
        assert_eq!(resolver.infer_provider("claude-3-sonnet").unwrap(), "anthropic");
        assert_eq!(resolver.infer_provider("deepseek-chat").unwrap(), "deepseek");
        assert_eq!(resolver.infer_provider("qwen-turbo").unwrap(), "qwen");
    }
    
    #[test]
    fn test_list_models() {
        let resolver = ModelResolver::new();
        let models = resolver.list_models();
        
        assert!(models.contains(&"gpt-4".to_string()));
        assert!(models.contains(&"claude-3-sonnet".to_string()));
        assert!(models.contains(&"deepseek-chat".to_string()));
    }
}
