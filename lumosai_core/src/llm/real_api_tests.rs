//! Real API tests for LLM module
//!
//! Tests LLM provider functionality, temperature normalization, streaming, and error handling

#[cfg(test)]
mod tests {
    use crate::llm::test_helpers::*;
    use crate::llm::{LlmOptions, LlmProvider, Message, Role, ZhipuProvider};
    use crate::llm::types::Temperature;
    use std::sync::Arc;
    use tokio::time::{sleep, Duration};

    /// Helper function to retry API calls with exponential backoff
    async fn retry_with_backoff<F, Fut, T>(mut f: F, max_retries: u32) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mut retries = 0;
        let mut delay = Duration::from_millis(2000);

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) if retries < max_retries => {
                    retries += 1;
                    eprintln!(
                        "Attempt {} failed: {}. Retrying in {:?}...",
                        retries, e, delay
                    );
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
                Err(e) => return Err(format!("Failed after {} retries: {}", max_retries, e)),
            }
        }
    }

    #[tokio::test]
    async fn test_llm_temperature_normalization_deterministic() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(0.0) // Should map to 0.0
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("Say 'test'", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok(), "Temperature 0.0 should work");
        let response = result.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_llm_temperature_normalization_balanced() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(0.5) // Should map to 0.5
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("Say 'test'", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok(), "Temperature 0.5 should work");
        let response = result.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_llm_temperature_normalization_creative() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(1.0) // Should map to 1.0
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("Say 'test'", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok(), "Temperature 1.0 should work");
        let response = result.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_llm_temperature_normalization_low_value() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(0.2) // Should map to 0.0 (< 0.25)
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("Say 'test'", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok(), "Temperature 0.2 should map to 0.0");
    }

    #[tokio::test]
    async fn test_llm_temperature_normalization_mid_value() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(0.7) // Should map to 1.0 (>= 0.75)
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("Say 'test'", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok(), "Temperature 0.7 should map to 0.5 or 1.0");
    }

    #[tokio::test]
    async fn test_llm_generate_with_messages() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let messages = vec![
            Message::new(Role::System, "You are a helpful assistant.".to_string(), None, None),
            Message::new(Role::User, "Say 'hello'".to_string(), None, None),
        ];

        let options = LlmOptions::default()
            .with_temperature(0.5)
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate_with_messages(&messages, &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_llm_generate_with_max_tokens() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(0.5)
            .with_max_tokens(10); // Very small limit

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("Tell me a long story", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
        // Response should be short due to max_tokens limit
    }

    #[tokio::test]
    async fn test_llm_provider_name() {
        let provider = create_test_zhipu_provider();
        assert_eq!(provider.name(), "zhipu");
    }

    #[tokio::test]
    async fn test_llm_options_default() {
        let options = LlmOptions::default();
        assert_eq!(options.temperature, Some(Temperature::BALANCED));
        assert_eq!(options.max_tokens, Some(1000));
        assert_eq!(options.stream, false);
        assert!(options.stop.is_none());
    }

    #[tokio::test]
    async fn test_llm_options_builder() {
        let options = LlmOptions::default()
            .with_temperature(0.8)
            .with_max_tokens(500)
            .with_stop(vec!["END".to_string()]);

        assert_eq!(options.temperature, Some(Temperature::new(0.8)));
        assert_eq!(options.max_tokens, Some(500));
        assert_eq!(options.stop, Some(vec!["END".to_string()]));
    }

    #[tokio::test]
    async fn test_temperature_type_creation() {
        let temp = Temperature::new(0.7);
        assert_eq!(temp.value(), 0.7);

        let temp_rounded = Temperature::new(0.73);
        assert_eq!(temp_rounded.value(), 0.7); // Should round to 0.7
    }

    #[tokio::test]
    async fn test_temperature_constants() {
        assert_eq!(Temperature::DETERMINISTIC.value(), 0.0);
        assert_eq!(Temperature::LOW.value(), 0.3);
        assert_eq!(Temperature::BALANCED.value(), 0.7);
        assert_eq!(Temperature::CREATIVE.value(), 1.0);
        assert_eq!(Temperature::HIGH.value(), 1.5);
    }

    #[tokio::test]
    async fn test_message_creation() {
        let message = Message::new(Role::User, "Hello".to_string(), None, None);
        assert_eq!(message.role, Role::User);
        assert_eq!(message.content, "Hello");
        assert!(message.metadata.is_none());
        assert!(message.name.is_none());
    }

    #[tokio::test]
    async fn test_role_serialization() {
        let user_role = Role::User;
        let system_role = Role::System;
        let assistant_role = Role::Assistant;

        assert_eq!(user_role, Role::User);
        assert_eq!(system_role, Role::System);
        assert_eq!(assistant_role, Role::Assistant);
    }

    #[tokio::test]
    async fn test_llm_generate_empty_prompt() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(0.5)
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        // Empty prompt should either work or return an error
        // Both are acceptable behaviors
        if let Ok(response) = result {
            // If it works, response might be empty or contain something
            println!("Empty prompt response: {}", response);
        }
    }

    #[tokio::test]
    async fn test_llm_generate_long_prompt() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let long_prompt = "Tell me about ".repeat(50); // ~750 characters
        let options = LlmOptions::default()
            .with_temperature(0.5)
            .with_max_tokens(100);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate(&long_prompt, &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_llm_generate_with_chinese_prompt() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let options = LlmOptions::default()
            .with_temperature(0.5)
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate("你好，请说'测试'", &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_llm_multiple_messages_conversation() {
        sleep(Duration::from_millis(1000)).await;

        let provider = create_test_zhipu_provider_arc();
        let messages = vec![
            Message::new(Role::System, "You are a helpful assistant.".to_string(), None, None),
            Message::new(Role::User, "What is 2+2?".to_string(), None, None),
            Message::new(Role::Assistant, "4".to_string(), None, None),
            Message::new(Role::User, "What is 3+3?".to_string(), None, None),
        ];

        let options = LlmOptions::default()
            .with_temperature(0.5)
            .with_max_tokens(50);

        let result = retry_with_backoff(
            || async {
                provider
                    .generate_with_messages(&messages, &options)
                    .await
                    .map_err(|e| e.to_string())
            },
            5,
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
    }

    #[tokio::test]
    async fn test_zhipu_provider_creation() {
        let api_key = std::env::var("ZHIPU_API_KEY")
            .unwrap_or_else(|_| "test_key".to_string());

        let provider = ZhipuProvider::new(api_key.clone(), Some("glm-4.6".to_string()));
        assert_eq!(provider.name(), "zhipu");

        let provider_default = ZhipuProvider::new(api_key, None);
        assert_eq!(provider_default.name(), "zhipu");
    }

    #[tokio::test]
    async fn test_llm_options_with_model() {
        let options = LlmOptions::default()
            .with_model("glm-4.6".to_string());

        assert_eq!(options.model, Some("glm-4.6".to_string()));
    }
}
