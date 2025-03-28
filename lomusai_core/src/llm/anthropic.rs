use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{Error, Result};
use super::provider::LlmProvider;
use super::types::{LlmOptions, Message};

/// Anthropic API响应结构
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic消息请求结构
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic LLM provider implementation
pub struct AnthropicProvider {
    /// Anthropic API密钥
    pub(crate) api_key: String,
    /// 使用的模型名称
    pub(crate) model: String,
    /// Anthropic API基础URL
    base_url: String,
    /// HTTP客户端
    client: reqwest::Client,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(api_key: String, model: String) -> Self {
        Self { 
            api_key, 
            model,
            base_url: "https://api.anthropic.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    /// 设置自定义的API基础URL
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
    
    /// 创建授权请求头
    fn create_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", 
            HeaderValue::from_str(&self.api_key).expect("Invalid API key format")
        );
        headers.insert("anthropic-version", 
            HeaderValue::from_static("2023-06-01")
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }
    
    /// 从Lomusai消息转换为Anthropic消息格式
    fn convert_messages(&self, messages: &[Message]) -> Vec<AnthropicMessage> {
        messages
            .iter()
            .map(|msg| {
                // Anthropic只支持user和assistant角色，system消息需要特殊处理
                let role = if msg.role == "system" {
                    "user".to_string()
                } else {
                    msg.role.clone()
                };
                
                AnthropicMessage {
                    role,
                    content: msg.content.clone(),
                }
            })
            .collect()
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        let url = format!("{}/messages", self.base_url);
        
        // 准备消息
        let messages = if let Some(messages) = &options.messages {
            self.convert_messages(messages)
        } else {
            vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }]
        };
        
        // 创建请求
        let request = AnthropicRequest {
            model: self.model.clone(),
            messages,
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            top_p: None,
            stream: None,
        };
        
        // 发送请求
        let response = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| Error::LlmError(format!("Anthropic API request failed: {}", e)))?;
            
        // 检查响应状态
        let status = response.status();
        
        if !status.is_success() {
            // 如果响应不成功，获取错误文本
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
                
            return Err(Error::LlmError(format!(
                "Anthropic API error: {}, details: {}",
                status,
                error_text
            )));
        }
        
        // 如果响应成功，解析JSON
        let response_data: AnthropicResponse = response.json().await
            .map_err(|e| Error::LlmError(format!("Failed to parse Anthropic response: {}", e)))?;
            
        // 提取生成的文本
        let generated_text = response_data.content
            .iter()
            .filter(|content| content.content_type == "text")
            .map(|content| content.text.clone())
            .collect::<Vec<String>>()
            .join("");
            
        Ok(generated_text)
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
        // Anthropic目前不提供官方的嵌入API，我们返回一个错误
        Err(Error::LlmError("Anthropic does not provide an embedding API".to_string()))
    }
} 