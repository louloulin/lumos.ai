use async_trait::async_trait;
use futures::stream;
use futures::stream::BoxStream;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::Result;
use crate::Error;
use super::provider::LlmProvider;
use super::types::{LlmOptions, Message};

/// OpenAI compatible API response structure
#[derive(Debug, Deserialize)]
struct OpenAICompatResponse {
    choices: Vec<OpenAICompatChoice>,
    #[serde(default)]
    usage: Option<OpenAICompatUsage>,
}

/// OpenAI compatible API choice structure
#[derive(Debug, Deserialize)]
struct OpenAICompatChoice {
    message: OpenAICompatMessage,
    finish_reason: Option<String>,
}

/// OpenAI compatible API message structure
#[derive(Debug, Deserialize)]
struct OpenAICompatMessage {
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAICompatToolCall>>,
}

/// OpenAI compatible API tool call structure
#[derive(Debug, Deserialize)]
struct OpenAICompatToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAICompatFunction,
}

/// OpenAI compatible API function structure
#[derive(Debug, Deserialize)]
struct OpenAICompatFunction {
    name: String,
    arguments: String,
}

/// OpenAI compatible API usage structure
#[derive(Debug, Deserialize)]
struct OpenAICompatUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

/// OpenAI compatible API embedding response structure
#[derive(Debug, Deserialize)]
struct OpenAICompatEmbeddingResponse {
    data: Vec<OpenAICompatEmbeddingData>,
    usage: OpenAICompatEmbeddingUsage,
}

/// OpenAI compatible API embedding data structure
#[derive(Debug, Deserialize)]
struct OpenAICompatEmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

/// OpenAI compatible API embedding usage structure
#[derive(Debug, Deserialize)]
struct OpenAICompatEmbeddingUsage {
    prompt_tokens: u32,
    total_tokens: u32,
}

/// DashScope API response structure
#[derive(Debug, Deserialize)]
struct DashScopeResponse {
    output: DashScopeOutput,
}

/// DashScope API output structure
#[derive(Debug, Deserialize)]
struct DashScopeOutput {
    text: Option<String>,
    choices: Option<Vec<DashScopeChoice>>,
    tool_calls: Option<Vec<DashScopeToolCall>>,
}

/// DashScope API choice structure
#[derive(Debug, Deserialize)]
struct DashScopeChoice {
    message: DashScopeMessage,
}

/// DashScope API message structure
#[derive(Debug, Deserialize)]
struct DashScopeMessage {
    content: String,
    tool_calls: Option<Vec<DashScopeToolCall>>,
}

/// DashScope API tool call structure
#[derive(Debug, Deserialize)]
struct DashScopeToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: DashScopeFunction,
}

/// DashScope API function structure
#[derive(Debug, Deserialize)]
struct DashScopeFunction {
    name: String,
    arguments: String,
}

/// DashScope API embedding response structure
#[derive(Debug, Deserialize)]
struct DashScopeEmbeddingResponse {
    output: DashScopeEmbeddingOutput,
}

/// DashScope API embedding output structure
#[derive(Debug, Deserialize)]
struct DashScopeEmbeddingOutput {
    embeddings: Vec<DashScopeEmbedding>,
}

/// DashScope API embedding structure
#[derive(Debug, Deserialize)]
struct DashScopeEmbedding {
    embedding: Vec<f32>,
}

/// Tool parameter schema for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolParameter {
    #[serde(rename = "type")]
    param_type: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
}

/// Tool properties for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolProperties {
    #[serde(flatten)]
    properties: serde_json::Map<String, serde_json::Value>,
}

/// Tool schema for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolSchema {
    #[serde(rename = "type")]
    schema_type: String,
    properties: ToolProperties,
    required: Vec<String>,
}

/// Tool definition for Qwen function calling
#[derive(Debug, Serialize)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolFunction,
}

/// Tool function for Qwen function calling
#[derive(Debug, Serialize)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: ToolSchema,
}

/// API type enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QwenApiType {
    /// DashScope API (Alibaba Cloud)
    DashScope,
    /// OpenAI compatible API
    OpenAICompatible,
}

/// Qwen LLM provider implementation
pub struct QwenProvider {
    /// API base URL
    base_url: String,
    /// API key
    api_key: String,
    /// HTTP client
    client: Client,
    /// Default model to use
    model: String,
    /// Embedding model to use
    embedding_model: String,
    /// API type
    api_type: QwenApiType,
}

impl QwenProvider {
    /// Create a new Qwen provider with custom base URL and API type
    pub fn new_with_api_type(api_key: impl Into<String>, model: impl Into<String>, base_url: impl Into<String>, api_type: QwenApiType) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            client: Client::new(),
            model: model.into(),
            embedding_model: match api_type {
                QwenApiType::DashScope => "text-embedding-v1".to_string(),
                QwenApiType::OpenAICompatible => "text-embedding-ada-002".to_string(),
            },
            api_type,
        }
    }

    /// Create a new Qwen provider with DashScope API
    pub fn new(api_key: impl Into<String>, model: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self::new_with_api_type(api_key, model, base_url, QwenApiType::DashScope)
    }

    /// Create a new Qwen provider with default DashScope base URL
    pub fn new_with_defaults(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new_with_api_type(
            api_key,
            model,
            "https://dashscope.aliyuncs.com/api/v1",
            QwenApiType::DashScope
        )
    }

    /// Create a new Qwen provider configured for Qwen 2.5 models with DashScope API
    pub fn new_qwen25(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self::new_with_api_type(
            api_key,
            model,
            "https://dashscope.aliyuncs.com/api/v1",
            QwenApiType::DashScope
        )
    }

    /// Create a new Qwen provider with OpenAI-compatible API (for vLLM or similar servers)
    pub fn new_openai_compatible(api_key: impl Into<String>, model: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self::new_with_api_type(
            api_key,
            model,
            base_url,
            QwenApiType::OpenAICompatible
        )
    }

    /// Create request headers with API key
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        match self.api_type {
            QwenApiType::DashScope => {
                headers.insert(
                    AUTHORIZATION, 
                    HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                        .expect("Invalid API key format")
                );
            },
            QwenApiType::OpenAICompatible => {
                headers.insert(
                    "x-api-key", 
                    HeaderValue::from_str(&self.api_key)
                        .expect("Invalid API key format")
                );
                // Some OpenAI-compatible APIs require this header
                if !self.api_key.is_empty() && self.api_key != "EMPTY" {
                    headers.insert(
                        AUTHORIZATION, 
                        HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                            .expect("Invalid API key format")
                    );
                }
            }
        }

        headers
    }
}

#[async_trait]
impl LlmProvider for QwenProvider {
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        // Convert prompt to message format
        let messages = vec![Message {
            role: "user".into(),
            content: prompt.to_string(),
            metadata: None,
            name: None,
        }];
        
        self.generate_with_messages(&messages, options).await
    }
    
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        match self.api_type {
            QwenApiType::DashScope => self.generate_with_messages_dashscope(messages, options).await,
            QwenApiType::OpenAICompatible => self.generate_with_messages_openai(messages, options).await,
        }
    }
    
    async fn generate_stream<'a>(
        &'a self, 
        prompt: &'a str, 
        options: &'a LlmOptions
    ) -> Result<BoxStream<'a, Result<String>>> {
        // 简单实现，实际API流式处理需要更复杂的处理逻辑
        let response = self.generate(prompt, options).await?;
        
        // 创建一个简单的流，返回完整响应
        let stream = stream::once(async move { Ok(response) });
        
        Ok(Box::pin(stream))
    }
    
    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        match self.api_type {
            QwenApiType::DashScope => self.get_embedding_dashscope(text).await,
            QwenApiType::OpenAICompatible => self.get_embedding_openai(text).await,
        }
    }
}

// DashScope API implementation
impl QwenProvider {
    async fn generate_with_messages_dashscope(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        // 准备请求数据
        let url = format!("{}/services/aigc/text-generation/generation", self.base_url);
        
        // 转换消息格式
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                let mut message = serde_json::json!({
                    "role": msg.role.as_str(),
                    "content": msg.content.clone(),
                });
                
                // 如果有名称，添加名称
                if let Some(name) = &msg.name {
                    message["name"] = serde_json::json!(name);
                }
                
                message
            })
            .collect();
        
        // 构建请求正文
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "input": {
                "messages": api_messages,
            },
            "parameters": {}
        });
        
        // 添加选项参数
        if let Some(temperature) = options.temperature {
            body["parameters"]["temperature"] = serde_json::json!(temperature);
        }
        
        if let Some(max_tokens) = options.max_tokens {
            body["parameters"]["max_tokens"] = serde_json::json!(max_tokens);
        }
        
        if let Some(stop) = &options.stop {
            body["parameters"]["stop"] = serde_json::json!(stop);
        }
        
        // 添加工具定义（如果在额外参数中提供）
        if let Some(tools) = options.extra.get("tools") {
            body["input"]["tools"] = tools.clone();
        }
        
        // 添加 tool_choice（如果在额外参数中提供）
        if let Some(tool_choice) = options.extra.get("tool_choice") {
            body["input"]["tool_choice"] = tool_choice.clone();
        }
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Qwen API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read Qwen response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "Qwen API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: DashScopeResponse = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse Qwen response: {}: {}", e, text)))?;
            
        // 检查是否有工具调用
        if let Some(tool_calls) = &response.output.tool_calls {
            if !tool_calls.is_empty() {
                // 返回第一个工具调用的信息
                let tool_call = &tool_calls[0];
                let result = format!(
                    "Function: {}\nArguments: {}", 
                    tool_call.function.name, 
                    tool_call.function.arguments
                );
                return Ok(result);
            }
        }
        
        // 如果没有工具调用，提取生成的文本
        let content = response.output.text
            .ok_or_else(|| Error::Llm("No text content in Qwen response".to_string()))?;
            
        Ok(content)
    }

    async fn get_embedding_dashscope(&self, text: &str) -> Result<Vec<f32>> {
        // 准备请求数据
        let url = format!("{}/services/embeddings/text-embedding/v1/embeddings", self.base_url);
        
        let body = serde_json::json!({
            "model": self.embedding_model,
            "input": {
                "texts": [text]
            }
        });
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Qwen API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read Qwen embedding response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "Qwen API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: DashScopeEmbeddingResponse = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse Qwen embedding response: {}: {}", e, text)))?;
            
        // 提取嵌入向量
        if response.output.embeddings.is_empty() {
            return Err(Error::Llm("No embeddings returned from Qwen".to_string()));
        }
        
        let embedding = response.output.embeddings[0].embedding.clone();
        
        Ok(embedding)
    }
}

// OpenAI-compatible API implementation
impl QwenProvider {
    async fn generate_with_messages_openai(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        // OpenAI-compatible API endpoint
        let url = format!("{}/v1/chat/completions", self.base_url);
        
        // 转换消息格式
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                let mut message = serde_json::json!({
                    "role": msg.role.as_str(),
                    "content": msg.content.clone(),
                });
                
                // 如果有名称，添加名称
                if let Some(name) = &msg.name {
                    message["name"] = serde_json::json!(name);
                }
                
                message
            })
            .collect();
        
        // 构建请求正文
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "messages": api_messages,
        });
        
        // 添加选项参数
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }
        
        if let Some(stop) = &options.stop {
            body["stop"] = serde_json::json!(stop);
        }
        
        // 添加工具定义（如果在额外参数中提供）
        if let Some(tools) = options.extra.get("tools") {
            body["tools"] = tools.clone();
        }
        
        // 添加 tool_choice（如果在额外参数中提供）
        if let Some(tool_choice) = options.extra.get("tool_choice") {
            body["tool_choice"] = tool_choice.clone();
        }
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Qwen API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read Qwen response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "Qwen API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: OpenAICompatResponse = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse Qwen response: {}: {}", e, text)))?;
            
        if response.choices.is_empty() {
            return Err(Error::Llm("No choices returned from Qwen".to_string()));
        }
        
        let choice = &response.choices[0];
        
        // 检查是否有工具调用
        if let Some(tool_calls) = &choice.message.tool_calls {
            if !tool_calls.is_empty() {
                // 返回第一个工具调用的信息
                let tool_call = &tool_calls[0];
                let result = format!(
                    "Function: {}\nArguments: {}", 
                    tool_call.function.name, 
                    tool_call.function.arguments
                );
                return Ok(result);
            }
        }
        
        // 如果没有工具调用，提取生成的文本
        let content = choice.message.content
            .as_ref()
            .ok_or_else(|| Error::Llm("No content in message".to_string()))?;
            
        Ok(content.clone())
    }
    
    async fn get_embedding_openai(&self, text: &str) -> Result<Vec<f32>> {
        // OpenAI-compatible API endpoint
        let url = format!("{}/v1/embeddings", self.base_url);
        
        let body = serde_json::json!({
            "model": self.embedding_model,
            "input": text
        });
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Qwen API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read Qwen embedding response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "Qwen API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: OpenAICompatEmbeddingResponse = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse Qwen embedding response: {}: {}", e, text)))?;
            
        // 提取嵌入向量
        if response.data.is_empty() {
            return Err(Error::Llm("No embeddings returned from Qwen".to_string()));
        }
        
        let embedding = response.data[0].embedding.clone();
        
        Ok(embedding)
    }
} 