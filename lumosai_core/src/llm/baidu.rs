use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use futures::TryStreamExt;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::Value;

use crate::{Error, Result};
use super::provider::{LlmProvider, FunctionCallingResponse};
use super::types::{LlmOptions, Message};
use super::function_calling::{FunctionDefinition, FunctionCall, ToolChoice};

/// 百度ERNIE API response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaiduResponse {
    result: String,
    #[serde(default)]
    usage: Option<BaiduUsage>,
    id: String,
    object: String,
    created: u64,
    #[serde(default)]
    function_call: Option<BaiduFunctionCall>,
}

/// 百度ERNIE streaming response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaiduStreamResponse {
    result: Option<String>,
    #[serde(default)]
    usage: Option<BaiduUsage>,
    id: String,
    object: String,
    created: u64,
    is_end: Option<bool>,
    #[serde(default)]
    function_call: Option<BaiduFunctionCall>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaiduUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct BaiduFunctionCall {
    name: String,
    arguments: String,
}

/// 百度ERNIE embedding response structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaiduEmbeddingResponse {
    data: Vec<BaiduEmbeddingData>,
    usage: BaiduEmbeddingUsage,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaiduEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
    object: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaiduEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// 百度ERNIE access token response
#[derive(Debug, Deserialize)]
struct BaiduTokenResponse {
    access_token: String,
    expires_in: u64,
}

/// 百度ERNIE LLM provider
#[derive(Clone)]
pub struct BaiduProvider {
    api_key: String,
    secret_key: String,
    client: reqwest::Client,
    model: String,
    base_url: String,
    access_token: Option<String>,
}

impl BaiduProvider {
    /// Create a new 百度ERNIE provider
    pub fn new(api_key: String, secret_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            secret_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "ernie-bot".to_string()),
            base_url: "https://aip.baidubce.com".to_string(),
            access_token: None,
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

    /// Create a new 百度ERNIE provider with custom base URL
    pub fn with_base_url(api_key: String, secret_key: String, base_url: String, model: Option<String>) -> Self {
        Self {
            api_key,
            secret_key,
            client: reqwest::Client::new(),
            model: model.unwrap_or_else(|| "ernie-bot".to_string()),
            base_url,
            access_token: None,
        }
    }

    /// Get access token for 百度ERNIE API
    async fn get_access_token(&mut self) -> Result<String> {
        if let Some(token) = &self.access_token {
            return Ok(token.clone());
        }

        let url = format!(
            "{}/oauth/2.0/token?grant_type=client_credentials&client_id={}&client_secret={}",
            self.base_url, self.api_key, self.secret_key
        );

        let res = self.client
            .post(&url)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("百度ERNIE token request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 百度ERNIE token response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "百度ERNIE token API returned error status {}: {}",
                status, text
            )));
        }

        let token_response: BaiduTokenResponse = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse 百度ERNIE token response: {}", e)))?;

        self.access_token = Some(token_response.access_token.clone());
        Ok(token_response.access_token)
    }

    /// Create HTTP headers for 百度ERNIE API requests
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    /// Convert internal Message to 百度ERNIE API format
    fn convert_messages(&self, messages: &[Message]) -> Vec<Value> {
        messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": msg.role.as_str(),
                    "content": msg.content.clone(),
                })
            })
            .collect()
    }

    /// Get the appropriate endpoint for the model
    fn get_model_endpoint(&self, model: &str) -> &str {
        match model {
            "ernie-bot" => "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions",
            "ernie-bot-turbo" => "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/eb-instant",
            "ernie-bot-4" => "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions_pro",
            "ernie-3.5" => "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions",
            _ => "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions",
        }
    }
}

#[async_trait]
impl LlmProvider for BaiduProvider {
    fn name(&self) -> &str {
        "baidu"
    }

    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        // Convert prompt to messages format
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": prompt
        })];

        let mut provider = self.clone();
        let access_token = provider.get_access_token().await?;
        
        let model = options.model.clone().unwrap_or_else(|| self.model.clone());
        let endpoint = self.get_model_endpoint(&model);
        let url = format!("{}{}?access_token={}", self.base_url, endpoint, access_token);
        
        // Build request body
        let mut body = serde_json::json!({
            "messages": messages,
        });

        // Add optional parameters
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_output_tokens"] = serde_json::json!(max_tokens);
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
            .map_err(|e| Error::Llm(format!("百度ERNIE API request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 百度ERNIE response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "百度ERNIE API returned error status {}: {}",
                status, text
            )));
        }
        
        // Parse response
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse 百度ERNIE response: {}", e)))?;
            
        // Extract generated text
        let content = response["result"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from 百度ERNIE".to_string()))?;
            
        Ok(content.to_string())
    }
    
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        let mut provider = self.clone();
        let access_token = provider.get_access_token().await?;
        
        let model = options.model.clone().unwrap_or_else(|| self.model.clone());
        let endpoint = self.get_model_endpoint(&model);
        let url = format!("{}{}?access_token={}", self.base_url, endpoint, access_token);
        
        // Convert messages to 百度ERNIE format
        let api_messages = self.convert_messages(messages);
        
        // Build request body
        let mut body = serde_json::json!({
            "messages": api_messages,
        });
        
        // Add optional parameters
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_output_tokens"] = serde_json::json!(max_tokens);
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
            .map_err(|e| Error::Llm(format!("百度ERNIE API request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 百度ERNIE response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "百度ERNIE API returned error status {}: {}",
                status, text
            )));
        }
        
        // Parse response
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse 百度ERNIE response: {}", e)))?;
            
        // Extract generated text
        let content = response["result"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from 百度ERNIE".to_string()))?;
            
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

        let mut provider = self.clone();
        let access_token = provider.get_access_token().await?;

        let model = options.model.clone().unwrap_or_else(|| self.model.clone());
        let endpoint = self.get_model_endpoint(&model);
        let url = format!("{}{}?access_token={}", self.base_url, endpoint, access_token);

        // Build request body with streaming enabled
        let mut body = serde_json::json!({
            "messages": messages,
            "stream": true,
        });

        // Add optional parameters
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }

        if let Some(max_tokens) = options.max_tokens {
            body["max_output_tokens"] = serde_json::json!(max_tokens);
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
            .map_err(|e| Error::Llm(format!("百度ERNIE streaming request failed: {}", e)))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::Llm(format!(
                "百度ERNIE streaming API returned error status {}: {}",
                status, error_text
            )));
        }

        // Create stream from response body
        let stream = self.create_sse_stream(response).await?;
        Ok(Box::pin(stream))
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let mut provider = self.clone();
        let access_token = provider.get_access_token().await?;

        let url = format!("{}/rpc/2.0/ai_custom/v1/wenxinworkshop/embeddings/embedding-v1?access_token={}",
                         self.base_url, access_token);

        let body = serde_json::json!({
            "input": [text],
        });

        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("百度ERNIE embedding request failed: {}", e)))?;

        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 百度ERNIE embedding response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "百度ERNIE embedding API returned error status {}: {}",
                status, text
            )));
        }

        let response: BaiduEmbeddingResponse = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse 百度ERNIE embedding response: {}", e)))?;

        if response.data.is_empty() {
            return Err(Error::Llm("No embedding data in 百度ERNIE response".to_string()));
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
        _tool_choice: &ToolChoice,
        options: &LlmOptions,
    ) -> Result<FunctionCallingResponse> {
        let mut provider = self.clone();
        let access_token = provider.get_access_token().await?;

        let model = options.model.clone().unwrap_or_else(|| self.model.clone());
        let endpoint = self.get_model_endpoint(&model);
        let url = format!("{}{}?access_token={}", self.base_url, endpoint, access_token);

        // Convert messages to 百度ERNIE format
        let api_messages = self.convert_messages(messages);

        // Convert function definitions to 百度ERNIE functions format
        let functions_json: Vec<Value> = functions.iter().map(|func| {
            serde_json::json!({
                "name": func.name,
                "description": func.description,
                "parameters": func.parameters
            })
        }).collect();

        // Build request
        let mut body = serde_json::json!({
            "messages": api_messages,
        });

        // Add functions if provided
        if !functions_json.is_empty() {
            body["functions"] = Value::Array(functions_json);
        }

        // Add other options
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        if let Some(max_tokens) = options.max_tokens {
            body["max_output_tokens"] = serde_json::json!(max_tokens);
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
            .map_err(|e| Error::Llm(format!("百度ERNIE API request failed: {}", e)))?;

        let status = res.status();
        let response_text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read 百度ERNIE response: {}", e)))?;

        if !status.is_success() {
            return Err(Error::Llm(format!(
                "百度ERNIE API returned error status {}: {}",
                status, response_text
            )));
        }

        // Parse response
        let response: BaiduResponse = serde_json::from_str(&response_text)
            .map_err(|e| Error::Llm(format!("Failed to parse 百度ERNIE response: {}", e)))?;

        // Convert function calls
        let function_calls: Vec<FunctionCall> = if let Some(function_call) = response.function_call {
            vec![FunctionCall {
                id: None, // 百度ERNIE doesn't provide function call IDs
                name: function_call.name,
                arguments: function_call.arguments,
            }]
        } else {
            Vec::new()
        };

        Ok(FunctionCallingResponse {
            content: Some(response.result),
            function_calls,
            finish_reason: "stop".to_string(),
        })
    }
}

impl BaiduProvider {
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
                            match serde_json::from_str::<BaiduStreamResponse>(data) {
                                Ok(stream_response) => {
                                    // Check if this is the end of the stream
                                    if stream_response.is_end == Some(true) {
                                        break;
                                    }

                                    if let Some(result) = stream_response.result {
                                        if !result.is_empty() {
                                            results.push(result);
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(Error::Llm(format!(
                                        "Failed to parse 百度ERNIE streaming response: {}", e
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
    fn test_baidu_provider_creation() {
        let provider = BaiduProvider::new("test-key".to_string(), "test-secret".to_string(), None);
        assert_eq!(provider.model, "ernie-bot");
        assert_eq!(provider.base_url, "https://aip.baidubce.com");
    }

    #[test]
    fn test_baidu_provider_with_custom_model() {
        let provider = BaiduProvider::new("test-key".to_string(), "test-secret".to_string(), Some("ernie-bot-4".to_string()));
        assert_eq!(provider.model, "ernie-bot-4");
    }

    #[test]
    fn test_baidu_provider_with_custom_base_url() {
        let provider = BaiduProvider::with_base_url(
            "test-key".to_string(),
            "test-secret".to_string(),
            "https://custom.api.example.com".to_string(),
            None
        );
        assert_eq!(provider.base_url, "https://custom.api.example.com");
    }

    #[test]
    fn test_supports_function_calling() {
        let provider = BaiduProvider::new("test-key".to_string(), "test-secret".to_string(), None);
        assert!(provider.supports_function_calling());
    }

    #[test]
    fn test_provider_name() {
        let provider = BaiduProvider::new("test-key".to_string(), "test-secret".to_string(), None);
        assert_eq!(provider.name(), "baidu");
    }

    #[test]
    fn test_get_model_endpoint() {
        let provider = BaiduProvider::new("test-key".to_string(), "test-secret".to_string(), None);

        assert_eq!(provider.get_model_endpoint("ernie-bot"), "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions");
        assert_eq!(provider.get_model_endpoint("ernie-bot-turbo"), "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/eb-instant");
        assert_eq!(provider.get_model_endpoint("ernie-bot-4"), "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions_pro");
        assert_eq!(provider.get_model_endpoint("unknown-model"), "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/completions");
    }
}
