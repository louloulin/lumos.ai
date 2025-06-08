//! Together AI LLM provider implementation
//! 
//! This module provides integration with Together AI for accessing open-source models.

use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::error::{Error, Result};
use super::{
    LlmProvider, LlmOptions, Message, Role,
    function_calling::{FunctionDefinition, ToolChoice},
    provider::FunctionCallingResponse
};

/// Together AI API configuration
#[derive(Debug, Clone)]
pub struct TogetherConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub embedding_model: String,
}

impl Default for TogetherConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("TOGETHER_API_KEY").unwrap_or_default(),
            base_url: "https://api.together.xyz".to_string(),
            model: "meta-llama/Llama-2-7b-chat-hf".to_string(),
            embedding_model: "togethercomputer/m2-bert-80M-8k-retrieval".to_string(),
        }
    }
}

/// Together AI message format (OpenAI compatible)
#[derive(Debug, Serialize, Deserialize)]
pub struct TogetherMessage {
    pub role: String,
    pub content: String,
}

/// Together AI chat completion request
#[derive(Debug, Serialize)]
pub struct TogetherChatRequest {
    pub model: String,
    pub messages: Vec<TogetherMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Together AI completion request
#[derive(Debug, Serialize)]
pub struct TogetherCompletionRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

/// Together AI response
#[derive(Debug, Deserialize)]
pub struct TogetherResponse {
    pub choices: Vec<TogetherChoice>,
}

#[derive(Debug, Deserialize)]
pub struct TogetherChoice {
    pub message: Option<TogetherMessage>,
    pub text: Option<String>,
    pub finish_reason: Option<String>,
}

/// Together AI embedding request
#[derive(Debug, Serialize)]
pub struct TogetherEmbeddingRequest {
    pub model: String,
    pub input: String,
}

/// Together AI embedding response
#[derive(Debug, Deserialize)]
pub struct TogetherEmbeddingResponse {
    pub data: Vec<TogetherEmbeddingData>,
}

#[derive(Debug, Deserialize)]
pub struct TogetherEmbeddingData {
    pub embedding: Vec<f32>,
}

/// Together AI LLM provider
#[derive(Debug, Clone)]
pub struct TogetherProvider {
    config: TogetherConfig,
    client: Client,
}

impl TogetherProvider {
    /// Create a new Together provider
    pub fn new(api_key: String, model: String) -> Self {
        let config = TogetherConfig {
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
        let api_key = std::env::var("TOGETHER_API_KEY")
            .map_err(|_| Error::Configuration("TOGETHER_API_KEY environment variable not set".to_string()))?;
        
        let model = std::env::var("TOGETHER_MODEL")
            .unwrap_or_else(|_| "meta-llama/Llama-2-7b-chat-hf".to_string());
            
        Ok(Self::new(api_key, model))
    }

    /// Convert messages to Together format
    fn convert_messages(&self, messages: &[Message]) -> Vec<TogetherMessage> {
        messages
            .iter()
            .map(|msg| TogetherMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Function | Role::Tool | Role::Custom(_) => {
                        // For unsupported roles, treat as user message
                        "user".to_string()
                    }
                },
                content: msg.content.clone(),
            })
            .collect()
    }
}

#[async_trait]
impl LlmProvider for TogetherProvider {
    fn name(&self) -> &str {
        "together"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let request = TogetherCompletionRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.extra.get("top_p").and_then(|v| v.as_f64()).map(|f| f as f32),
            top_k: None,
            stop: options.stop.clone(),
        };

        let response = self.client
            .post(&format!("{}/v1/completions", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Together API error: {}", error_text)));
        }

        let response_json: TogetherResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        if response_json.choices.is_empty() {
            return Err(Error::Parsing("No choices in response".to_string()));
        }

        let choice = &response_json.choices[0];
        choice.text
            .as_ref()
            .map(|text| text.trim().to_string())
            .ok_or_else(|| Error::Parsing("No text in choice".to_string()))
    }

    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let together_messages = self.convert_messages(messages);
        
        let request = TogetherChatRequest {
            model: self.config.model.clone(),
            messages: together_messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            top_p: options.extra.get("top_p").and_then(|v| v.as_f64()).map(|f| f as f32),
            top_k: None,
            stop: options.stop.clone(),
            stream: Some(false),
        };

        let response = self.client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Together API error: {}", error_text)));
        }

        let response_json: TogetherResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        if response_json.choices.is_empty() {
            return Err(Error::Parsing("No choices in response".to_string()));
        }

        let choice = &response_json.choices[0];
        choice.message
            .as_ref()
            .map(|msg| msg.content.clone())
            .ok_or_else(|| Error::Parsing("No message in choice".to_string()))
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        // For now, return a simple stream with the full response
        // TODO: Implement proper streaming when needed
        let response = self.generate(prompt, options).await?;
        let stream = futures::stream::once(async move { Ok(response) });
        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = TogetherEmbeddingRequest {
            model: self.config.embedding_model.clone(),
            input: text.to_string(),
        };

        let response = self.client
            .post(&format!("{}/v1/embeddings", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Together API error: {}", error_text)));
        }

        let response_json: TogetherEmbeddingResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        if response_json.data.is_empty() {
            return Err(Error::Parsing("No embedding data in response".to_string()));
        }

        Ok(response_json.data[0].embedding.clone())
    }

    fn supports_function_calling(&self) -> bool {
        false // Together AI doesn't support OpenAI-style function calling yet
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
    fn test_together_provider_creation() {
        let provider = TogetherProvider::new(
            "test-key".to_string(),
            "meta-llama/Llama-2-7b-chat-hf".to_string(),
        );
        
        assert_eq!(provider.name(), "together");
        assert_eq!(provider.config.model, "meta-llama/Llama-2-7b-chat-hf");
    }

    #[test]
    fn test_message_conversion() {
        let provider = TogetherProvider::new(
            "test-key".to_string(),
            "meta-llama/Llama-2-7b-chat-hf".to_string(),
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
        
        let together_messages = provider.convert_messages(&messages);
        assert_eq!(together_messages.len(), 2);
        assert_eq!(together_messages[0].role, "system");
        assert_eq!(together_messages[1].role, "user");
        assert_eq!(together_messages[0].content, "You are a helpful assistant.");
        assert_eq!(together_messages[1].content, "Hello!");
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_together_integration() {
        let api_key = std::env::var("TOGETHER_API_KEY").expect("TOGETHER_API_KEY not set");
        let provider = TogetherProvider::new(api_key, "meta-llama/Llama-2-7b-chat-hf".to_string());
        
        let options = LlmOptions::default()
            .with_temperature(0.7)
            .with_max_tokens(50);
        
        let response = provider.generate("Say hello", &options).await;
        assert!(response.is_ok());
        println!("Together response: {}", response.unwrap());
    }
}
