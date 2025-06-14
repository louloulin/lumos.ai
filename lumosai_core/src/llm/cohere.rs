//! Cohere LLM provider implementation
//! 
//! This module provides integration with Cohere's language models and embedding models.

use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde_json::json;

use crate::error::{Error, Result};
use super::{
    LlmProvider, LlmOptions, Message, Role,
    function_calling::{FunctionDefinition, ToolChoice},
    provider::FunctionCallingResponse
};

/// Cohere API configuration
#[derive(Debug, Clone)]
pub struct CohereConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub embedding_model: String,
}

impl Default for CohereConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("COHERE_API_KEY").unwrap_or_default(),
            base_url: "https://api.cohere.ai".to_string(),
            model: "command-r-plus".to_string(),
            embedding_model: "embed-english-v3.0".to_string(),
        }
    }
}

/// Cohere LLM provider
#[derive(Debug, Clone)]
pub struct CohereProvider {
    config: CohereConfig,
    client: Client,
}

impl CohereProvider {
    /// Create a new Cohere provider
    pub fn new(api_key: String, model: String) -> Self {
        let config = CohereConfig {
            api_key,
            model,
            ..Default::default()
        };
        
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    "Authorization",
                    format!("Bearer {}", config.api_key)
                        .parse()
                        .expect("Invalid API key format"),
                );
                headers.insert(
                    "Content-Type",
                    "application/json".parse().unwrap(),
                );
                headers
            })
            .build()
            .expect("Failed to build HTTP client");

        Self { config, client }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("COHERE_API_KEY")
            .map_err(|_| Error::Configuration("COHERE_API_KEY environment variable not set".to_string()))?;
        
        let model = std::env::var("COHERE_MODEL")
            .unwrap_or_else(|_| "command-r-plus".to_string());
            
        Ok(Self::new(api_key, model))
    }

    /// Convert messages to Cohere format
    fn convert_messages(&self, messages: &[Message]) -> Result<String> {
        // Cohere uses a simple prompt format for now
        // In the future, we can implement proper conversation format
        let mut prompt = String::new();
        
        for message in messages {
            match message.role {
                Role::System => {
                    prompt.push_str(&format!("System: {}\n", message.content));
                }
                Role::User => {
                    prompt.push_str(&format!("Human: {}\n", message.content));
                }
                Role::Assistant => {
                    prompt.push_str(&format!("Assistant: {}\n", message.content));
                }
                Role::Function | Role::Tool | Role::Custom(_) => {
                    // For unsupported roles, treat as user message
                    prompt.push_str(&format!("Human: {}\n", message.content));
                }
            }
        }
        
        // Add final prompt for assistant response
        prompt.push_str("Assistant: ");
        
        Ok(prompt)
    }
}

#[async_trait]
impl LlmProvider for CohereProvider {
    fn name(&self) -> &str {
        "cohere"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let request_body = json!({
            "model": self.config.model,
            "prompt": prompt,
            "max_tokens": options.max_tokens.unwrap_or(1000),
            "temperature": options.temperature.unwrap_or(0.7),
            "k": 0,
            "stop_sequences": options.stop.as_ref().unwrap_or(&vec![]),
            "return_likelihoods": "NONE"
        });

        let response = self.client
            .post(&format!("{}/v1/generate", self.config.base_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Cohere API error: {}", error_text)));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        let text = response_json["generations"][0]["text"]
            .as_str()
            .ok_or_else(|| Error::Parsing("No text in response".to_string()))?;

        Ok(text.trim().to_string())
    }

    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let prompt = self.convert_messages(messages)?;
        self.generate(&prompt, options).await
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        // For now, return a simple stream with the full response
        // TODO: Implement proper streaming when Cohere supports it
        let response = self.generate(prompt, options).await?;
        let stream = futures::stream::once(async move { Ok(response) });
        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request_body = json!({
            "model": self.config.embedding_model,
            "texts": [text],
            "input_type": "search_document"
        });

        let response = self.client
            .post(&format!("{}/v1/embed", self.config.base_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Cohere API error: {}", error_text)));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        let embeddings = response_json["embeddings"][0]
            .as_array()
            .ok_or_else(|| Error::Parsing("No embeddings in response".to_string()))?;

        let embedding: Result<Vec<f32>> = embeddings
            .iter()
            .map(|v| {
                v.as_f64()
                    .map(|f| f as f32)
                    .ok_or_else(|| Error::Parsing("Invalid embedding value".to_string()))
            })
            .collect();

        embedding
    }

    fn supports_function_calling(&self) -> bool {
        false // Cohere doesn't support OpenAI-style function calling yet
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        _functions: &[FunctionDefinition],
        _tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        // Fallback to regular generation
        let content = self.generate_with_messages(messages, options).await?;
        Ok(FunctionCallingResponse {
            content: Some(content),
            function_calls: Vec::new(),
            finish_reason: "stop".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cohere_provider_creation() {
        let provider = CohereProvider::new(
            "test-key".to_string(),
            "command-r-plus".to_string(),
        );
        
        assert_eq!(provider.name(), "cohere");
        assert_eq!(provider.config.model, "command-r-plus");
    }

    #[test]
    fn test_message_conversion() {
        let provider = CohereProvider::new(
            "test-key".to_string(),
            "command-r-plus".to_string(),
        );
        
        let messages = vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant.".to_string(),
                metadata: None,
                name: None,
            },
            Message {
                role: Role::User,
                content: "Hello!".to_string(),
                metadata: None,
                name: None,
            },
        ];
        
        let prompt = provider.convert_messages(&messages).unwrap();
        assert!(prompt.contains("System: You are a helpful assistant."));
        assert!(prompt.contains("Human: Hello!"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_cohere_integration() {
        let api_key = std::env::var("COHERE_API_KEY").expect("COHERE_API_KEY not set");
        let provider = CohereProvider::new(api_key, "command-r-plus".to_string());
        
        let options = LlmOptions::default()
            .with_temperature(0.7)
            .with_max_tokens(50);
        
        let response = provider.generate("Say hello", &options).await;
        assert!(response.is_ok());
        println!("Cohere response: {}", response.unwrap());
    }
}
