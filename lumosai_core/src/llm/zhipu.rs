use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use futures::TryStreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::Value;

use crate::{Error, Result};
use super::provider::{LlmProvider, FunctionCallingResponse};
use super::types::{LlmOptions, Message, Role};
use super::function_calling::{FunctionDefinition, FunctionCall, ToolChoice};

/// 智谱AI API response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuResponse {
    choices: Vec<ZhipuChoice>,
    #[serde(default)]
    usage: Option<ZhipuUsage>,
    id: String,
    created: u64,
    model: String,
}

/// 智谱AI streaming response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuStreamResponse {
    choices: Vec<ZhipuStreamChoice>,
    #[serde(default)]
    usage: Option<ZhipuUsage>,
    id: String,
    created: u64,
    model: String,
}

#[derive(Debug, Deserialize)]
struct ZhipuStreamChoice {
    index: u32,
    delta: ZhipuStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuStreamDelta {
    role: Option<String>,
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ZhipuToolCall>,
}

#[derive(Debug, Deserialize)]
struct ZhipuChoice {
    index: u32,
    message: ZhipuMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuMessage {
    role: String,
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ZhipuToolCall>,
}

#[derive(Debug, Deserialize)]
struct ZhipuToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: ZhipuFunction,
}

#[derive(Debug, Deserialize)]
struct ZhipuFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// 智谱AI embedding response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuEmbeddingResponse {
    data: Vec<ZhipuEmbeddingData>,
    model: String,
    usage: ZhipuEmbeddingUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    object: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ZhipuEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// 智谱AI LLM provider
pub struct ZhipuProvider {
    api_key: String,
    client: reqwest::Client,
    model: String,
    base_url: String,
}

impl ZhipuProvider {
    /// Create a new 智谱AI provider
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "glm-4".to_string()),
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
        }
    }

    /// Get the model name
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Create a new 智谱AI provider with custom base URL
    pub fn with_base_url(api_key: String, base_url: String, model: Option<String>) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "glm-4".to_string()),
            base_url,
        }
    }

    /// Create HTTP headers for 智谱AI API requests
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

    /// Convert internal Message to 智谱AI API format
    fn convert_messages(&self, messages: &[Message]) -> Vec<Value> {
        messages
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

                // Add tool_call_id for tool messages
                if msg.role == Role::Tool {
                    if let Some(metadata) = &msg.metadata {
                        if let Some(tool_call_id) = metadata.get("tool_call_id") {
                            message["tool_call_id"] = tool_call_id.clone();
                        }
                    }
                }

                // Add tool_calls for assistant messages
                if msg.role == Role::Assistant {
                    if let Some(metadata) = &msg.metadata {
                        if let Some(tool_calls) = metadata.get("tool_calls") {
                            message["tool_calls"] = tool_calls.clone();
                        }
                    }
                }

                message
            })
            .collect()
    }
}

#[async_trait]
impl LlmProvider for ZhipuProvider {
    fn name(&self) -> &str {
        "zhipu"
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
            .map_err(|e| Error::Llm(format!("智谱AI API request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 智谱AI response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "智谱AI API returned error status {}: {}",
                status, text
            )));
        }
        
        // Parse response
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse 智谱AI response: {}", e)))?;
            
        // Extract generated text
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from 智谱AI".to_string()))?;
            
        Ok(content.to_string())
    }
    
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        // Prepare request data
        let url = format!("{}/chat/completions", self.base_url);
        
        // Convert messages to 智谱AI format
        let api_messages = self.convert_messages(messages);
        
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
            .map_err(|e| Error::Llm(format!("智谱AI API request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 智谱AI response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "智谱AI API returned error status {}: {}",
                status, text
            )));
        }
        
        // Parse response
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse 智谱AI response: {}", e)))?;
            
        // Extract generated text
        let content = response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from 智谱AI".to_string()))?;
            
        Ok(content.to_string())
    }

    async fn generate_stream<'a>(
        &'a self,
        prompt: &'a str,
        options: &'a LlmOptions
    ) -> Result<BoxStream<'a, Result<String>>> {
        // Convert prompt to messages format
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })];

        // Prepare request data
        let url = format!("{}/chat/completions", self.base_url);

        // Build request body with streaming enabled
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": messages,
            "stream": true,
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
        let response = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("智谱AI streaming request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Llm(format!(
                "智谱AI streaming API returned error status {}: {}",
                status, error_text
            )));
        }

        // Create stream from response body
        let stream = self.create_sse_stream(response).await?;
        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.base_url);

        let body = serde_json::json!({
            "model": "embedding-2",
            "input": text,
        });

        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("智谱AI embedding request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 智谱AI embedding response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "智谱AI embedding API returned error status {}: {}",
                status, text
            )));
        }

        let response: ZhipuEmbeddingResponse = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse 智谱AI embedding response: {}", e)))?;

        if response.data.is_empty() {
            return Err(Error::Llm("No embedding data in 智谱AI response".to_string()));
        }

        Ok(response.data[0].embedding.clone())
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

        // Convert messages to 智谱AI format
        let api_messages = self.convert_messages(messages);

        // Convert function definitions to 智谱AI tools format
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

        // Convert tool choice
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
            .map_err(|e| Error::Llm(format!("智谱AI API request failed: {}", e)))?;

        let status = res.status();
        let response_text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 智谱AI response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "智谱AI API returned error status {}: {}",
                status, response_text
            )));
        }

        // Parse response
        let response: ZhipuResponse = serde_json::from_str(&response_text)
            .map_err(|e| Error::Llm(format!("Failed to parse 智谱AI response: {}", e)))?;

        if response.choices.is_empty() {
            return Err(Error::Llm("No choices in 智谱AI response".to_string()));
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

impl ZhipuProvider {
    /// Create SSE stream from HTTP response
    async fn create_sse_stream(
        &self,
        response: reqwest::Response,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let byte_stream = response.bytes_stream();

        Ok(byte_stream
            .map_err(|e| Error::Llm(format!("HTTP stream error: {}", e)))
            .map(|chunk_result| {
                chunk_result.and_then(|chunk| {
                    // Convert bytes to string
                    let text = String::from_utf8(chunk.to_vec())
                        .map_err(|e| Error::Llm(format!("UTF-8 decode error: {}", e)))?;

                    // Split by lines and process each line
                    let mut results = Vec::new();
                    for line in text.lines() {
                        // Skip empty lines and comments
                        if line.trim().is_empty() || line.starts_with(':') {
                            continue;
                        }

                        // Parse SSE format: "data: {...}"
                        if let Some(data) = line.strip_prefix("data: ") {
                            // Handle end of stream
                            if data.trim() == "[DONE]" {
                                break;
                            }

                            // Parse JSON response
                            match serde_json::from_str::<ZhipuStreamResponse>(data) {
                                Ok(stream_response) => {
                                    if let Some(choice) = stream_response.choices.first() {
                                        if let Some(content) = &choice.delta.content {
                                            if !content.is_empty() {
                                                results.push(content.clone());
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(Error::Llm(format!(
                                        "Failed to parse 智谱AI streaming response: {}", e
                                    )));
                                }
                            }
                        }
                    }

                    // Join all content from this chunk
                    Ok(results.join(""))
                })
            })
            .filter_map(|result| async move {
                match result {
                    Ok(content) if !content.is_empty() => Some(Ok(content)),
                    Ok(_) => None, // Skip empty content
                    Err(e) => Some(Err(e)),
                }
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zhipu_provider_creation() {
        let provider = ZhipuProvider::new("test-key".to_string(), None);
        assert_eq!(provider.model, "glm-4");
        assert_eq!(provider.base_url, "https://open.bigmodel.cn/api/paas/v4");
    }

    #[test]
    fn test_zhipu_provider_with_custom_model() {
        let provider = ZhipuProvider::new("test-key".to_string(), Some("glm-4-plus".to_string()));
        assert_eq!(provider.model, "glm-4-plus");
    }

    #[test]
    fn test_zhipu_provider_with_custom_base_url() {
        let provider = ZhipuProvider::with_base_url(
            "test-key".to_string(),
            "https://custom.api.example.com".to_string(),
            None
        );
        assert_eq!(provider.base_url, "https://custom.api.example.com");
    }

    #[test]
    fn test_supports_function_calling() {
        let provider = ZhipuProvider::new("test-key".to_string(), None);
        assert!(provider.supports_function_calling());
    }

    #[test]
    fn test_provider_name() {
        let provider = ZhipuProvider::new("test-key".to_string(), None);
        assert_eq!(provider.name(), "zhipu");
    }
}
