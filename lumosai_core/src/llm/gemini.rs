//! Google Gemini LLM provider implementation
//! 
//! This module provides integration with Google's Gemini language models.

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

/// Gemini API configuration
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub embedding_model: String,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("GEMINI_API_KEY").unwrap_or_default(),
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            model: "gemini-1.5-pro".to_string(),
            embedding_model: "text-embedding-004".to_string(),
        }
    }
}

/// Gemini content part for multimodal support
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPart {
    pub text: String,
}

/// Gemini content structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub role: String,
    pub parts: Vec<ContentPart>,
}

/// Gemini generation config
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
}

/// Gemini request structure
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

/// Gemini response structure
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub content: Content,
    pub finish_reason: Option<String>,
}

/// Gemini LLM provider
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    config: GeminiConfig,
    client: Client,
}

impl GeminiProvider {
    /// Create a new Gemini provider
    pub fn new(api_key: String, model: String) -> Self {
        let config = GeminiConfig {
            api_key,
            model,
            ..Default::default()
        };
        
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

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .map_err(|_| Error::Configuration("GEMINI_API_KEY environment variable not set".to_string()))?;
        
        let model = std::env::var("GEMINI_MODEL")
            .unwrap_or_else(|_| "gemini-1.5-pro".to_string());
            
        Ok(Self::new(api_key, model))
    }

    /// Convert messages to Gemini format
    fn convert_messages(&self, messages: &[Message]) -> Result<Vec<Content>> {
        let mut contents = Vec::new();
        
        for message in messages {
            let role = match message.role {
                Role::System => {
                    // Gemini doesn't have a system role, so we'll add it as user context
                    "user".to_string()
                }
                Role::User => "user".to_string(),
                Role::Assistant => "model".to_string(),
                Role::Function | Role::Tool | Role::Custom(_) => {
                    // For unsupported roles, treat as user message
                    "user".to_string()
                }
            };
            
            let content = Content {
                role,
                parts: vec![ContentPart {
                    text: message.content.clone(),
                }],
            };
            
            contents.push(content);
        }
        
        Ok(contents)
    }

    /// Convert single prompt to Gemini format
    fn convert_prompt(&self, prompt: &str) -> Vec<Content> {
        vec![Content {
            role: "user".to_string(),
            parts: vec![ContentPart {
                text: prompt.to_string(),
            }],
        }]
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let contents = self.convert_prompt(prompt);
        
        let generation_config = GenerationConfig {
            temperature: options.temperature,
            max_output_tokens: options.max_tokens,
            top_p: options.extra.get("top_p").and_then(|v| v.as_f64()).map(|f| f as f32),
            top_k: None,
        };

        let request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
        };

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.config.base_url, self.config.model, self.config.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Gemini API error: {}", error_text)));
        }

        let response_json: GeminiResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        if response_json.candidates.is_empty() {
            return Err(Error::Parsing("No candidates in response".to_string()));
        }

        let candidate = &response_json.candidates[0];
        if candidate.content.parts.is_empty() {
            return Err(Error::Parsing("No parts in candidate content".to_string()));
        }

        Ok(candidate.content.parts[0].text.clone())
    }

    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let contents = self.convert_messages(messages)?;
        
        let generation_config = GenerationConfig {
            temperature: options.temperature,
            max_output_tokens: options.max_tokens,
            top_p: options.extra.get("top_p").and_then(|v| v.as_f64()).map(|f| f as f32),
            top_k: None,
        };

        let request = GeminiRequest {
            contents,
            generation_config: Some(generation_config),
        };

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.config.base_url, self.config.model, self.config.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Gemini API error: {}", error_text)));
        }

        let response_json: GeminiResponse = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        if response_json.candidates.is_empty() {
            return Err(Error::Parsing("No candidates in response".to_string()));
        }

        let candidate = &response_json.candidates[0];
        if candidate.content.parts.is_empty() {
            return Err(Error::Parsing("No parts in candidate content".to_string()));
        }

        Ok(candidate.content.parts[0].text.clone())
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
        let request_body = json!({
            "model": format!("models/{}", self.config.embedding_model),
            "content": {
                "parts": [{"text": text}]
            }
        });

        let url = format!(
            "{}/v1beta/models/{}:embedContent?key={}",
            self.config.base_url, self.config.embedding_model, self.config.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| Error::Network(format!("Failed to send request: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::LlmProvider(format!("Gemini API error: {}", error_text)));
        }

        let response_json: serde_json::Value = response.json().await
            .map_err(|e| Error::Parsing(format!("Failed to parse response: {}", e)))?;

        let values = response_json["embedding"]["values"]
            .as_array()
            .ok_or_else(|| Error::Parsing("No embedding values in response".to_string()))?;

        let embedding: Result<Vec<f32>> = values
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
        true // Gemini supports function calling
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        _functions: &[FunctionDefinition],
        _tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        // For now, fallback to regular generation
        // TODO: Implement proper function calling support
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
    fn test_gemini_provider_creation() {
        let provider = GeminiProvider::new(
            "test-key".to_string(),
            "gemini-1.5-pro".to_string(),
        );
        
        assert_eq!(provider.name(), "gemini");
        assert_eq!(provider.config.model, "gemini-1.5-pro");
    }

    #[test]
    fn test_message_conversion() {
        let provider = GeminiProvider::new(
            "test-key".to_string(),
            "gemini-1.5-pro".to_string(),
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
        
        let contents = provider.convert_messages(&messages).unwrap();
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0].role, "user"); // System becomes user
        assert_eq!(contents[1].role, "user");
        assert_eq!(contents[0].parts[0].text, "You are a helpful assistant.");
        assert_eq!(contents[1].parts[0].text, "Hello!");
    }

    #[tokio::test]
    #[ignore] // Requires API key
    async fn test_gemini_integration() {
        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY not set");
        let provider = GeminiProvider::new(api_key, "gemini-1.5-pro".to_string());
        
        let options = LlmOptions::default()
            .with_temperature(0.7)
            .with_max_tokens(50);
        
        let response = provider.generate("Say hello", &options).await;
        assert!(response.is_ok());
        println!("Gemini response: {}", response.unwrap());
    }
}
