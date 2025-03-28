use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use reqwest::header::{HeaderMap, HeaderValue};

use crate::error::{Error, Result};
use crate::llm::provider::LlmProvider;
use crate::llm::types::{LlmOptions, Message, Role};
use futures::stream::BoxStream;

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
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        headers
    }
    
    /// 将消息转换为Anthropic兼容格式
    fn format_messages_for_anthropic(&self, messages: &[Message]) -> String {
        let mut result = String::new();
        
        for (i, msg) in messages.iter().enumerate() {
            // 处理第一个系统消息
            if i == 0 && msg.role == Role::System {
                result.push_str(&format!("{}\n\n", msg.content));
                continue;
            }
            
            // Anthropic的消息格式
            let role_str = match msg.role {
                Role::User => "Human",
                Role::Assistant => "Assistant",
                Role::System => continue, // 系统消息已在开头处理
                Role::Function => "Human", // Anthropic不支持函数角色
                Role::Custom(_) => "Human", // 自定义角色默认作为人类
            };
            
            result.push_str(&format!("{}: {}\n\n", role_str, msg.content));
        }
        
        // 确保最后添加Assistant:前缀
        if !result.trim_end().ends_with("Assistant:") {
            result.push_str("Assistant: ");
        }
        
        result
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate(&self, prompt: &str, options: &LlmOptions) -> Result<String> {
        // 构建完整提示
        let full_prompt = format!("Human: {}\n\nAssistant:", prompt);
        
        // 准备请求数据
        let url = format!("{}/complete", self.base_url);
        
        // 构建请求正文
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "prompt": full_prompt,
        });
        
        // 添加选项参数
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens_to_sample"] = serde_json::json!(max_tokens);
        }
        
        if let Some(stop) = &options.stop {
            body["stop_sequences"] = serde_json::json!(stop);
        }
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Anthropic API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read Anthropic response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "Anthropic API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse Anthropic response: {}", e)))?;
            
        // 提取生成的文本
        let content = response["completion"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from Anthropic".to_string()))?;
            
        Ok(content.trim().to_string())
    }
    
    async fn generate_with_messages(&self, messages: &[Message], options: &LlmOptions) -> Result<String> {
        if messages.is_empty() {
            return Err(Error::Llm("No messages provided for Anthropic".to_string()));
        }
        
        // 将消息格式化为Anthropic格式
        let formatted_prompt = self.format_messages_for_anthropic(messages);
        
        // 准备请求数据
        let url = format!("{}/complete", self.base_url);
        
        // 构建请求正文
        let mut body = serde_json::json!({
            "model": options.model.clone().unwrap_or_else(|| self.model.clone()),
            "prompt": formatted_prompt,
        });
        
        // 添加选项参数
        if let Some(temperature) = options.temperature {
            body["temperature"] = serde_json::json!(temperature);
        }
        
        if let Some(max_tokens) = options.max_tokens {
            body["max_tokens_to_sample"] = serde_json::json!(max_tokens);
        }
        
        if let Some(stop) = &options.stop {
            body["stop_sequences"] = serde_json::json!(stop);
        }
        
        // 发送请求
        let res = self.client
            .post(&url)
            .headers(self.create_headers())
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Llm(format!("Anthropic API request failed: {}", e)))?;
            
        let status = res.status();
        let text = res.text().await
            .map_err(|e| Error::Llm(format!("Failed to read Anthropic response: {}", e)))?;
            
        if !status.is_success() {
            return Err(Error::Llm(format!(
                "Anthropic API returned error status {}: {}",
                status, text
            )));
        }
        
        // 解析响应
        let response: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| Error::Llm(format!("Failed to parse Anthropic response: {}", e)))?;
            
        // 提取生成的文本
        let content = response["completion"]
            .as_str()
            .ok_or_else(|| Error::Llm("Invalid response format from Anthropic".to_string()))?;
            
        Ok(content.trim().to_string())
    }
    
    async fn generate_stream<'a>(&'a self, prompt: &'a str, _options: &'a LlmOptions) -> Result<BoxStream<'a, Result<String>>> {
        // Anthropic支持流式输出，但我们暂时不实现
        Err(Error::Llm("Stream generation not implemented for Anthropic".to_string()))
    }
    
    async fn get_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        // Anthropic目前不提供嵌入API
        Err(Error::Llm("Anthropic does not provide an embedding API".to_string()))
    }
} 