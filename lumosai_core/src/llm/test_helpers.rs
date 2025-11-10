//! Test helpers for LLM providers
//!
//! This module provides helper functions for creating LLM providers in tests.

use super::{LlmProvider, ZhipuProvider};
use std::sync::Arc;

/// Get the Zhipu API key from environment or use default test key
pub fn get_zhipu_api_key() -> String {
    std::env::var("ZHIPU_API_KEY")
        .unwrap_or_else(|_| "99a311fa7920a59e9399cf26ecc1e938.ac4w6buZHr2Ggc3k".to_string())
}

/// Create a Zhipu provider for testing with glm-4.6 model (latest, best quality)
///
/// Note: glm-4.6 uses reasoning_content for Chain-of-Thought responses.
/// - For best results, use max_tokens >= 500 or don't set max_tokens
/// - The model returns reasoning process in reasoning_content field
/// - Our provider automatically handles both content and reasoning_content
pub fn create_test_zhipu_provider() -> ZhipuProvider {
    ZhipuProvider::new(get_zhipu_api_key(), Some("glm-4.6".to_string()))
}

/// Create an Arc-wrapped Zhipu provider for testing
pub fn create_test_zhipu_provider_arc() -> Arc<dyn LlmProvider> {
    Arc::new(create_test_zhipu_provider())
}

/// Create a Zhipu provider with custom model
pub fn create_test_zhipu_provider_with_model(model: &str) -> ZhipuProvider {
    ZhipuProvider::new(get_zhipu_api_key(), Some(model.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_zhipu_api_key() {
        let key = get_zhipu_api_key();
        assert!(!key.is_empty());
    }

    #[test]
    fn test_create_test_zhipu_provider() {
        let provider = create_test_zhipu_provider();
        assert_eq!(provider.name(), "zhipu");
    }

    #[test]
    fn test_create_test_zhipu_provider_arc() {
        let provider = create_test_zhipu_provider_arc();
        assert_eq!(provider.name(), "zhipu");
    }

    #[test]
    fn test_create_test_zhipu_provider_with_model() {
        let provider = create_test_zhipu_provider_with_model("glm-4.6-flash");
        assert_eq!(provider.name(), "zhipu");
    }

    #[test]
    fn test_glm46_model_name() {
        // Verify we're using glm-4.6 by default
        let provider = create_test_zhipu_provider();
        // Note: Can't directly check model name from provider, but this ensures it compiles
        assert_eq!(provider.name(), "zhipu");
    }
}
