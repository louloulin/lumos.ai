//! Ollama LLM provider implementation
//! 
//! This module provides integration with Ollama for running local language models.

use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
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

/// Ollama API configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: std::env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama2".to_string()),
        }
    }
}

/// Ollama message format
#[derive(Debug, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

/// Ollama chat request
#[derive(Debug, Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

/// Ollama generation request
#[derive(Debug, Serialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

/// Ollama options
#[derive(Debug, Serialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
}

/// Ollama response
#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub message: Option<OllamaMessage>,
    pub response: Option<String>,
    pub done: bool,
}

/// Ollama embedding request
#[derive(Debug, Serialize)]
pub struct OllamaEmbeddingRequest {
    pub model: String,
    pub prompt: String,
}

/// Ollama embedding response
#[derive(Debug, Deserialize)]
pub struct OllamaEmbeddingResponse {
    pub embedding: Vec<f32>,
}

/// Ollama LLM provider
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    config: OllamaConfig,
    client: Client,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    pub fn new(base_url: String, model: String) -> Self {
        let config = OllamaConfig { base_url, model };
        
        let client = Client::builder()
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
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

    /// Create from environment variables or defaults
    pub fn from_env() -> Self {
        let config = OllamaConfig::default();
        Self::new(config.base_url, config.model)
    }

    /// Create with default localhost URL
    pub fn localhost(model: String) -> Self {
        Self::new("http://localhost:11434".to_string(), model)
    }

    /// Convert messages to Ollama format
    fn convert_messages(&self, messages: &[Message]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|msg| OllamaMessage {
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

    /// Convert LlmOptions to OllamaOptions
    fn convert_options(&self, options: &LlmOptions) -> OllamaOptions {
        OllamaOptions {
            temperature: options.temperature,
            top_p: options.extra.get("top_p").and_then(|v| v.as_f64()).map(|f| f as f32),
            top_k: None,
            num_predict: options.max_tokens,
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let request = OllamaGenerateRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: Some(false),
            options: Some(self.convert_options(options)),
        };

        let response = self.client
            .post(&format!("{}/api/generate", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Ollama API error: {}", error_text)));
        }

        let response_json: OllamaResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        response_json.response
            .ok_or_else(|| Error::Parsing("No response text in Ollama response".to_string()))
    }

    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let ollama_messages = self.convert_messages(messages);
        
        let request = OllamaChatRequest {
            model: self.config.model.clone(),
            messages: ollama_messages,
            stream: Some(false),
            options: Some(self.convert_options(options)),
        };

        let response = self.client
            .post(&format!("{}/api/chat", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Ollama API error: {}", error_text)));
        }

        let response_json: OllamaResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        response_json.message
            .map(|msg| msg.content)
            .ok_or_else(|| Error::Parsing("No message content in Ollama response".to_string()))
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions,
    ) -> Result<BoxStream<'a, Result<String>>> {
        let request = OllamaGenerateRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            stream: Some(true),
            options: Some(self.convert_options(options)),
        };

        let response = self.client
            .post(&format!("{}/api/generate", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Ollama API error: {}", error_text)));
        }

        let stream = response
            .bytes_stream()
            .map(|chunk_result| {
                chunk_result
                    .map_err(|e| Error::Network(format!("Stream error: {}", e)))
                    .and_then(|chunk| {
                        let text = String::from_utf8_lossy(&chunk);
                        for line in text.lines() {
                            if line.trim().is_empty() {
                                continue;
                            }
                            
                            match serde_json::from_str::<OllamaResponse>(line) {
                                Ok(response) => {
                                    if let Some(content) = response.response {
                                        return Ok(content);
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                        Ok(String::new())
                    })
            })
            .filter(|result| {
                futures::future::ready(match result {
                    Ok(content) => !content.is_empty(),
                    Err(_) => true,
                })
            });

        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = OllamaEmbeddingRequest {
            model: self.config.model.clone(),
            prompt: text.to_string(),
        };

        let response = self.client
            .post(&format!("{}/api/embeddings", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Ollama API error: {}", error_text)));
        }

        let response_json: OllamaEmbeddingResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        Ok(response_json.embedding)
    }

    fn supports_function_calling(&self) -> bool {
        false // Ollama doesn't support OpenAI-style function calling
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
    fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new(
            "http://localhost:11434".to_string(),
            "llama2".to_string(),
        );
        
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.config.model, "llama2");
    }

    #[test]
    fn test_message_conversion() {
        let provider = OllamaProvider::localhost("llama2".to_string());
        
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
        
        let ollama_messages = provider.convert_messages(&messages);
        assert_eq!(ollama_messages.len(), 2);
        assert_eq!(ollama_messages[0].role, "system");
        assert_eq!(ollama_messages[1].role, "user");
        assert_eq!(ollama_messages[0].content, "You are a helpful assistant.");
        assert_eq!(ollama_messages[1].content, "Hello!");
    }

    #[tokio::test]
    #[ignore] // Requires Ollama server running
    async fn test_ollama_integration() {
        let provider = OllamaProvider::localhost("llama2".to_string());
        
        let options = LlmOptions::default()
            .with_temperature(0.7)
            .with_max_tokens(50);
        
        let response = provider.generate("Say hello", &options).await;
        assert!(response.is_ok());
        println!("Ollama response: {}", response.unwrap());
    }
}
