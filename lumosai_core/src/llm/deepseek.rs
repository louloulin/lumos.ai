use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::Value;

use crate::{Error, Result};
use super::provider::{LlmProvider, FunctionCallingResponse};
use super::types::{LlmOptions, Message, Role};
use super::function_calling::{FunctionDefinition, FunctionCall, ToolChoice};

/// DeepSeek API response structures (compatible with OpenAI format)
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepSeekResponse {
    choices: Vec<DeepSeekChoice>,
    #[serde(default)]
    usage: Option<DeepSeekUsage>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekChoice {
    message: DeepSeekMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepSeekMessage {
    role: String,
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<DeepSeekToolCall>,
}

#[derive(Debug, Deserialize)]
struct DeepSeekToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: DeepSeekFunction,
}

#[derive(Debug, Deserialize)]
struct DeepSeekFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepSeekUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// DeepSeek embedding response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepSeekEmbeddingResponse {
    data: Vec<DeepSeekEmbeddingData>,
    usage: DeepSeekEmbeddingUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepSeekEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepSeekEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// DeepSeek LLM provider
pub struct DeepSeekProvider {
    api_key: String,
    client: reqwest::Client,
    model: String,
    base_url: String,
}

impl DeepSeekProvider {
    /// Create a new DeepSeek provider
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "deepseek-chat".to_string()),
            base_url: "https://api.deepseek.com".to_string(),
        }
    }

    /// Create a new DeepSeek provider with custom base URL
    pub fn with_base_url(api_key: String, base_url: String, model: Option<String>) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "deepseek-chat".to_string()),
            base_url,
        }
    }

    /// Create HTTP headers for DeepSeek API requests
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .expect("Invalid API key format"),
        );
        headers
    }
}

#[async_trait]
impl LlmProvider for DeepSeekProvider {
    fn name(&self) -> &str {
        "deepseek"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        // Convert prompt to messages format
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })];

        // Prepare request data
        let url = format!("{}/chat/completions", self.base_url);
        
        // Build request body
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": messages,
        });

        // Add optional parameters
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        // Check for top_p in extra parameters
        if let Some(top_p) = options.extra.get("top_p") {
            body["top_p"] = top_p.clone();
        }

        // Send request
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("DeepSeek API request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read DeepSeek response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "DeepSeek API returned error status {}: {}",
                status, text
            )));
        }
        
        // Parse response
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse DeepSeek response: {}", e)))?;
            
        // Extract generated text
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from DeepSeek".to_string()))?;
            
        Ok(content.to_string())
    }
    
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        // Prepare request data
        let url = format!("{}/chat/completions", self.base_url);
        
        // Convert messages to DeepSeek format
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": msg.role.as_str(),
                    "content": msg.content.clone(),
                    "name": msg.name.clone(),
                })
            })
            .collect();
        
        // Build request body
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": api_messages,
        });
        
        // Add optional parameters
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        // Check for top_p in extra parameters
        if let Some(top_p) = options.extra.get("top_p") {
            body["top_p"] = top_p.clone();
        }

        // Send request
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("DeepSeek API request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read DeepSeek response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "DeepSeek API returned error status {}: {}",
                status, text
            )));
        }
        
        // Parse response
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse DeepSeek response: {}", e)))?;
            
        // Extract generated text
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from DeepSeek".to_string()))?;
            
        Ok(content.to_string())
    }
    
    async fn generate_stream<'a>(
        &'a self, 
        prompt: &'a str, 
        options: &'a LlmOptions
    ) -> Result<BoxStream<'a, Result<String>>> {
        // For now, implement a simple chunked response
        // TODO: Implement proper streaming using SSE (Server-Sent Events)
        let result = self.generate(prompt, options).await?;
        
        // Split the result into chunks for simulation
        let words: Vec<&str> = result.split_whitespace().collect();
        let chunks: Vec<Result<String>> = words
            .chunks(3)
            .map(|chunk| Ok(chunk.join(" ") + " "))
            .collect();
        
        Ok(Box::pin(stream::iter(chunks)))
    }
    
    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        // Note: DeepSeek doesn't provide embedding API in their current offering
        // This is a placeholder implementation
        // In a real implementation, you might want to:
        // 1. Use a different embedding service
        // 2. Return an error indicating embeddings are not supported
        // 3. Implement a fallback mechanism
        
        Err(Error::Llm("DeepSeek does not provide embedding API. Consider using OpenAI or other providers for embeddings.".to_string()))
    }

    fn supports_function_calling(&self) -> bool {
        true
    }

    async fn generate_with_functions(
        &self,
        messages: &[Message],
        functions: &[FunctionDefinition],
        tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        
        // Convert messages to DeepSeek format
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                let mut message = serde_json::json!({
                    "role": msg.role.as_str(),
                    "content": msg.content.clone(),
                });

                // Add name if present
                if let Some(name) = &msg.name {
                    message["name"] = serde_json::Value::String(name.clone());
                }

                // Add tool_call_id for tool messages (required by DeepSeek API)
                if msg.role == Role::Tool {
                    if let Some(metadata) = &msg.metadata {
                        if let Some(tool_call_id) = metadata.get("tool_call_id") {
                            message["tool_call_id"] = tool_call_id.clone();
                        }
                    }
                }

                // Add tool_calls for assistant messages (if they contain function calls)
                if msg.role == Role::Assistant {
                    if let Some(metadata) = &msg.metadata {
                        if let Some(tool_calls) = metadata.get("tool_calls") {
                            message["tool_calls"] = tool_calls.clone();
                        }
                    }
                }

                message
            })
            .collect();

        // Convert function definitions to DeepSeek tools format (same as OpenAI)
        let tools: Vec<Value> = functions.iter().map(|func| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": func.name,
                    "description": func.description,
                    "parameters": func.parameters
                }
            })
        }).collect();

        // Convert tool choice (same format as OpenAI)
        let tool_choice_value = match tool_choice {
            ToolChoice::Auto => serde_json::json!("auto"),
            ToolChoice::None => serde_json::json!("none"),
            ToolChoice::Required => serde_json::json!("required"),
            ToolChoice::Function { name } => serde_json::json!({
                "type": "function",
                "function": { "name": name }
            }),
        };

        // Build request
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": api_messages,
        });

        // Add tools if provided
        if !tools.is_empty() {
            body["tools"] = Value::Array(tools);
            body["tool_choice"] = tool_choice_value;
        }

        // Add other options
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        if let Some(top_p) = options.extra.get("top_p") {
            body["top_p"] = top_p.clone();
        }

        // Send request
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("DeepSeek API request failed: {}", e)))?;

        let status = res.status();
        let response_text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read DeepSeek response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "DeepSeek API returned error status {}: {}",
                status, response_text
            )));
        }

        // Parse response
        let response: DeepSeekResponse = serde_json::from_str(&response_text)
            .map_err(|e| Error::Llm(format!("Failed to parse DeepSeek response: {}", e)))?;

        if response.choices.is_empty() {
            return Err(Error::Llm("No choices in DeepSeek response".to_string()));
        }

        let choice = &response.choices[0];
        let message = &choice.message;

        // Convert function calls
        let function_calls: Vec<FunctionCall> = message.tool_calls
            .iter()
            .filter(|tc| tc.call_type == "function")
            .map(|tc| FunctionCall {
                id: Some(tc.id.clone()),
                name: tc.function.name.clone(),
                arguments: tc.function.arguments.clone(),
            })
            .collect();

        Ok(FunctionCallingResponse {
            content: message.content.clone(),
            function_calls,
            finish_reason: choice.finish_reason.clone().unwrap_or_else(|| "stop".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_provider_creation() {
        let provider = DeepSeekProvider::new("test-key".to_string(), None);
        assert_eq!(provider.model, "deepseek-chat");
        assert_eq!(provider.base_url, "https://api.deepseek.com");
    }

    #[test]
    fn test_deepseek_provider_with_custom_model() {
        let provider = DeepSeekProvider::new("test-key".to_string(), Some("deepseek-reasoner".to_string()));
        assert_eq!(provider.model, "deepseek-reasoner");
    }

    #[test]
    fn test_deepseek_provider_with_custom_base_url() {
        let provider = DeepSeekProvider::with_base_url(
            "test-key".to_string(),
            "https://custom.api.example.com".to_string(),
            None
        );
        assert_eq!(provider.base_url, "https://custom.api.example.com");
    }

    #[test]
    fn test_supports_function_calling() {
        let provider = DeepSeekProvider::new("test-key".to_string(), None);
        assert!(provider.supports_function_calling());
    }

    #[tokio::test]
    async fn test_get_embedding_returns_error() {
        let provider = DeepSeekProvider::new("test-key".to_string(), None);
        let result = provider.get_embedding("test text").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("DeepSeek does not provide embedding API"));
    }
}
